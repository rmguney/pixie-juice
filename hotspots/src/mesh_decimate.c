//! Mesh Decimation using Quadric Error Metrics

#include "mesh_decimate.h"
#include "util.h"

// External WASM memory management
extern void* wasm_malloc(size_t size);
extern void wasm_free(void* ptr);

// Optimized WASM math functions
static inline float fast_sqrt(float x) {
    if (x <= 0.0f) return 0.0f;
    
    // Newton's method optimized for WASM
    float guess = x * 0.5f;
    for (int i = 0; i < 6; i++) {
        guess = (guess + x / guess) * 0.5f;
    }
    return guess;
}

static inline float fast_abs(float x) {
    return x < 0.0f ? -x : x;
}

// 4x4 Matrix operations for Quadric Error Metrics
typedef struct {
    float m[4][4];
} Matrix4;

static void matrix_zero(Matrix4* mat) {
    for (int i = 0; i < 4; i++) {
        for (int j = 0; j < 4; j++) {
            mat->m[i][j] = 0.0f;
        }
    }
}

static void matrix_add(Matrix4* result, const Matrix4* a, const Matrix4* b) {
    for (int i = 0; i < 4; i++) {
        for (int j = 0; j < 4; j++) {
            result->m[i][j] = a->m[i][j] + b->m[i][j];
        }
    }
}

static float matrix_evaluate_quadric(const Matrix4* mat, float x, float y, float z) {
    // Evaluate v^T * Q * v where v = [x, y, z, 1]
    float v[4] = {x, y, z, 1.0f};
    float result = 0.0f;
    
    for (int i = 0; i < 4; i++) {
        for (int j = 0; j < 4; j++) {
            result += v[i] * mat->m[i][j] * v[j];
        }
    }
    
    return result;
}

// Vertex structure for QEM
typedef struct {
    float pos[3];
    Matrix4 quadric;
    int valid;
    size_t* adjacent_vertices;
    size_t adjacent_count;
    size_t adjacent_capacity;
} QEMVertex;

// Edge structure for collapse operations
typedef struct {
    size_t v1, v2;
    float cost;
    float target_pos[3];
} QEMEdge;

// Priority queue for edge collapse (simple heap)
typedef struct {
    QEMEdge* edges;
    size_t count;
    size_t capacity;
} EdgeQueue;

static void edge_queue_init(EdgeQueue* queue, size_t capacity) {
    queue->edges = (QEMEdge*)wasm_malloc(capacity * sizeof(QEMEdge));
    queue->count = 0;
    queue->capacity = capacity;
}

static void edge_queue_free(EdgeQueue* queue) {
    wasm_free(queue->edges);
    queue->edges = NULL;
    queue->count = 0;
    queue->capacity = 0;
}

static void edge_queue_heapify_up(EdgeQueue* queue, size_t index) {
    while (index > 0) {
        size_t parent = (index - 1) / 2;
        if (queue->edges[index].cost >= queue->edges[parent].cost) break;
        
        QEMEdge temp = queue->edges[index];
        queue->edges[index] = queue->edges[parent];
        queue->edges[parent] = temp;
        
        index = parent;
    }
}

static void edge_queue_heapify_down(EdgeQueue* queue, size_t index) {
    while (index * 2 + 1 < queue->count) {
        size_t left = index * 2 + 1;
        size_t right = index * 2 + 2;
        size_t smallest = index;
        
        if (queue->edges[left].cost < queue->edges[smallest].cost) {
            smallest = left;
        }
        
        if (right < queue->count && queue->edges[right].cost < queue->edges[smallest].cost) {
            smallest = right;
        }
        
        if (smallest == index) break;
        
        QEMEdge temp = queue->edges[index];
        queue->edges[index] = queue->edges[smallest];
        queue->edges[smallest] = temp;
        
        index = smallest;
    }
}

static void edge_queue_push(EdgeQueue* queue, const QEMEdge* edge) {
    if (queue->count >= queue->capacity) return;
    
    queue->edges[queue->count] = *edge;
    edge_queue_heapify_up(queue, queue->count);
    queue->count++;
}

static int edge_queue_pop(EdgeQueue* queue, QEMEdge* edge) {
    if (queue->count == 0) return 0;
    
    *edge = queue->edges[0];
    queue->edges[0] = queue->edges[queue->count - 1];
    queue->count--;
    
    if (queue->count > 0) {
        edge_queue_heapify_down(queue, 0);
    }
    
    return 1;
}

// Calculate plane equation for triangle
static void calculate_plane(const float* v1, const float* v2, const float* v3, float plane[4]) {
    // Calculate normal using cross product
    float edge1[3] = {v2[0] - v1[0], v2[1] - v1[1], v2[2] - v1[2]};
    float edge2[3] = {v3[0] - v1[0], v3[1] - v1[1], v3[2] - v1[2]};
    
    float normal[3] = {
        edge1[1] * edge2[2] - edge1[2] * edge2[1],
        edge1[2] * edge2[0] - edge1[0] * edge2[2],
        edge1[0] * edge2[1] - edge1[1] * edge2[0]
    };
    
    // Normalize
    float length = fast_sqrt(normal[0]*normal[0] + normal[1]*normal[1] + normal[2]*normal[2]);
    if (length > 1e-6f) {
        normal[0] /= length;
        normal[1] /= length;
        normal[2] /= length;
    }
    
    // Calculate d coefficient
    plane[0] = normal[0];
    plane[1] = normal[1];
    plane[2] = normal[2];
    plane[3] = -(normal[0] * v1[0] + normal[1] * v1[1] + normal[2] * v1[2]);
}

// Create quadric matrix from plane equation
static void create_quadric_from_plane(Matrix4* quadric, const float plane[4]) {
    matrix_zero(quadric);
    
    for (int i = 0; i < 4; i++) {
        for (int j = 0; j < 4; j++) {
            quadric->m[i][j] = plane[i] * plane[j];
        }
    }
}

// Calculate optimal collapse position for edge
static float calculate_edge_collapse_cost(const QEMVertex* v1, const QEMVertex* v2, float target[3]) {
    // Combine quadrics
    Matrix4 combined;
    matrix_add(&combined, &v1->quadric, &v2->quadric);
    
    // For simplicity, use midpoint as target position
    target[0] = (v1->pos[0] + v2->pos[0]) * 0.5f;
    target[1] = (v1->pos[1] + v2->pos[1]) * 0.5f;
    target[2] = (v1->pos[2] + v2->pos[2]) * 0.5f;
    
    // Evaluate quadric error at target position
    return matrix_evaluate_quadric(&combined, target[0], target[1], target[2]);
}

// Add adjacent vertex to vertex adjacency list
static void add_adjacent_vertex(QEMVertex* vertex, size_t adjacent_index) {
    // Check if already exists
    for (size_t i = 0; i < vertex->adjacent_count; i++) {
        if (vertex->adjacent_vertices[i] == adjacent_index) {
            return;
        }
    }
    
    // Expand capacity if needed
    if (vertex->adjacent_count >= vertex->adjacent_capacity) {
        size_t new_capacity = vertex->adjacent_capacity * 2;
        if (new_capacity == 0) new_capacity = 8;
        
        size_t* new_adjacent = (size_t*)wasm_malloc(new_capacity * sizeof(size_t));
        if (!new_adjacent) return;
        
        if (vertex->adjacent_vertices) {
            for (size_t i = 0; i < vertex->adjacent_count; i++) {
                new_adjacent[i] = vertex->adjacent_vertices[i];
            }
            wasm_free(vertex->adjacent_vertices);
        }
        
        vertex->adjacent_vertices = new_adjacent;
        vertex->adjacent_capacity = new_capacity;
    }
    
    vertex->adjacent_vertices[vertex->adjacent_count++] = adjacent_index;
}

// Main QEM decimation function
MeshDecimateResult decimate_mesh_qem(const float* vertices, size_t vertex_count,
                                    const uint32_t* indices, size_t index_count,
                                    float target_ratio) {
    MeshDecimateResult result = {0};
    
    if (!vertices || !indices || vertex_count == 0 || index_count == 0) {
        result.success = 0;
        const char* msg = "Invalid input parameters";
        for (int i = 0; i < 255 && msg[i]; i++) {
            result.error_message[i] = msg[i];
        }
        return result;
    }
    
    if (target_ratio <= 0.0f || target_ratio >= 1.0f) {
        result.success = 0;
        const char* msg = "Target ratio must be between 0 and 1";
        for (int i = 0; i < 255 && msg[i]; i++) {
            result.error_message[i] = msg[i];
        }
        return result;
    }
    
    size_t target_vertex_count = (size_t)(vertex_count * target_ratio);
    if (target_vertex_count < 3) target_vertex_count = 3;
    
    // Initialize QEM vertices
    QEMVertex* qem_vertices = (QEMVertex*)wasm_malloc(vertex_count * sizeof(QEMVertex));
    if (!qem_vertices) {
        result.success = 0;
        const char* msg = "Memory allocation failed";
        for (int i = 0; i < 255 && msg[i]; i++) {
            result.error_message[i] = msg[i];
        }
        return result;
    }
    
    for (size_t i = 0; i < vertex_count; i++) {
        qem_vertices[i].pos[0] = vertices[i * 3];
        qem_vertices[i].pos[1] = vertices[i * 3 + 1];
        qem_vertices[i].pos[2] = vertices[i * 3 + 2];
        qem_vertices[i].valid = 1;
        qem_vertices[i].adjacent_vertices = NULL;
        qem_vertices[i].adjacent_count = 0;
        qem_vertices[i].adjacent_capacity = 0;
        matrix_zero(&qem_vertices[i].quadric);
    }
    
    // Build adjacency information and initial quadrics
    size_t triangle_count = index_count / 3;
    for (size_t t = 0; t < triangle_count; t++) {
        uint32_t i1 = indices[t * 3];
        uint32_t i2 = indices[t * 3 + 1];
        uint32_t i3 = indices[t * 3 + 2];
        
        if (i1 >= vertex_count || i2 >= vertex_count || i3 >= vertex_count) {
            continue;
        }
        
        // Add adjacency relationships
        add_adjacent_vertex(&qem_vertices[i1], i2);
        add_adjacent_vertex(&qem_vertices[i1], i3);
        add_adjacent_vertex(&qem_vertices[i2], i1);
        add_adjacent_vertex(&qem_vertices[i2], i3);
        add_adjacent_vertex(&qem_vertices[i3], i1);
        add_adjacent_vertex(&qem_vertices[i3], i2);
        
        // Calculate plane equation for triangle
        float plane[4];
        calculate_plane(qem_vertices[i1].pos, qem_vertices[i2].pos, qem_vertices[i3].pos, plane);
        
        // Create quadric for this plane
        Matrix4 face_quadric;
        create_quadric_from_plane(&face_quadric, plane);
        
        // Add to vertex quadrics
        matrix_add(&qem_vertices[i1].quadric, &qem_vertices[i1].quadric, &face_quadric);
        matrix_add(&qem_vertices[i2].quadric, &qem_vertices[i2].quadric, &face_quadric);
        matrix_add(&qem_vertices[i3].quadric, &qem_vertices[i3].quadric, &face_quadric);
    }
    
    // Initialize edge queue
    EdgeQueue edge_queue;
    edge_queue_init(&edge_queue, vertex_count * 6); // Estimate edge count
    
    // Add all edges to queue with collapse costs
    for (size_t i = 0; i < vertex_count; i++) {
        if (!qem_vertices[i].valid) continue;
        
        for (size_t j = 0; j < qem_vertices[i].adjacent_count; j++) {
            size_t adjacent = qem_vertices[i].adjacent_vertices[j];
            if (adjacent > i && qem_vertices[adjacent].valid) { // Avoid duplicates
                QEMEdge edge;
                edge.v1 = i;
                edge.v2 = adjacent;
                edge.cost = calculate_edge_collapse_cost(&qem_vertices[i], &qem_vertices[adjacent], edge.target_pos);
                edge_queue_push(&edge_queue, &edge);
            }
        }
    }
    
    // Perform edge collapses
    size_t current_vertex_count = vertex_count;
    while (current_vertex_count > target_vertex_count && edge_queue.count > 0) {
        QEMEdge edge;
        if (!edge_queue_pop(&edge_queue, &edge)) break;
        
        // Check if vertices are still valid
        if (!qem_vertices[edge.v1].valid || !qem_vertices[edge.v2].valid) {
            continue;
        }
        
        // Collapse edge: merge v2 into v1
        qem_vertices[edge.v1].pos[0] = edge.target_pos[0];
        qem_vertices[edge.v1].pos[1] = edge.target_pos[1];
        qem_vertices[edge.v1].pos[2] = edge.target_pos[2];
        
        // Combine quadrics
        matrix_add(&qem_vertices[edge.v1].quadric, &qem_vertices[edge.v1].quadric, &qem_vertices[edge.v2].quadric);
        
        // Mark v2 as invalid
        qem_vertices[edge.v2].valid = 0;
        current_vertex_count--;
        
        // Update adjacency for v1 (merge adjacency lists)
        for (size_t i = 0; i < qem_vertices[edge.v2].adjacent_count; i++) {
            size_t adjacent = qem_vertices[edge.v2].adjacent_vertices[i];
            if (adjacent != edge.v1 && qem_vertices[adjacent].valid) {
                add_adjacent_vertex(&qem_vertices[edge.v1], adjacent);
            }
        }
    }
    
    // Build output mesh
    size_t* vertex_map = (size_t*)wasm_malloc(vertex_count * sizeof(size_t));
    float* new_vertices = (float*)wasm_malloc(current_vertex_count * 3 * sizeof(float));
    
    if (!vertex_map || !new_vertices) {
        wasm_free(vertex_map);
        wasm_free(new_vertices);
        edge_queue_free(&edge_queue);
        
        // Free adjacency lists
        for (size_t i = 0; i < vertex_count; i++) {
            wasm_free(qem_vertices[i].adjacent_vertices);
        }
        wasm_free(qem_vertices);
        
        result.success = 0;
        const char* msg = "Output allocation failed";
        for (int i = 0; i < 255 && msg[i]; i++) {
            result.error_message[i] = msg[i];
        }
        return result;
    }
    
    // Create vertex mapping and copy valid vertices
    size_t new_vertex_index = 0;
    for (size_t i = 0; i < vertex_count; i++) {
        if (qem_vertices[i].valid) {
            vertex_map[i] = new_vertex_index;
            new_vertices[new_vertex_index * 3] = qem_vertices[i].pos[0];
            new_vertices[new_vertex_index * 3 + 1] = qem_vertices[i].pos[1];
            new_vertices[new_vertex_index * 3 + 2] = qem_vertices[i].pos[2];
            new_vertex_index++;
        } else {
            vertex_map[i] = SIZE_MAX; // Invalid mapping
        }
    }
    
    // Count valid triangles and create index buffer
    size_t valid_triangle_count = 0;
    for (size_t t = 0; t < triangle_count; t++) {
        uint32_t i1 = indices[t * 3];
        uint32_t i2 = indices[t * 3 + 1];
        uint32_t i3 = indices[t * 3 + 2];
        
        if (i1 < vertex_count && i2 < vertex_count && i3 < vertex_count &&
            qem_vertices[i1].valid && qem_vertices[i2].valid && qem_vertices[i3].valid) {
            valid_triangle_count++;
        }
    }
    
    uint32_t* new_indices = (uint32_t*)wasm_malloc(valid_triangle_count * 3 * sizeof(uint32_t));
    if (!new_indices) {
        wasm_free(vertex_map);
        wasm_free(new_vertices);
        edge_queue_free(&edge_queue);
        
        for (size_t i = 0; i < vertex_count; i++) {
            wasm_free(qem_vertices[i].adjacent_vertices);
        }
        wasm_free(qem_vertices);
        
        result.success = 0;
        const char* msg = "Index allocation failed";
        for (int i = 0; i < 255 && msg[i]; i++) {
            result.error_message[i] = msg[i];
        }
        return result;
    }
    
    size_t new_index_count = 0;
    for (size_t t = 0; t < triangle_count; t++) {
        uint32_t i1 = indices[t * 3];
        uint32_t i2 = indices[t * 3 + 1];
        uint32_t i3 = indices[t * 3 + 2];
        
        if (i1 < vertex_count && i2 < vertex_count && i3 < vertex_count &&
            qem_vertices[i1].valid && qem_vertices[i2].valid && qem_vertices[i3].valid) {
            new_indices[new_index_count * 3] = (uint32_t)vertex_map[i1];
            new_indices[new_index_count * 3 + 1] = (uint32_t)vertex_map[i2];
            new_indices[new_index_count * 3 + 2] = (uint32_t)vertex_map[i3];
            new_index_count++;
        }
    }
    
    // Cleanup
    wasm_free(vertex_map);
    edge_queue_free(&edge_queue);
    for (size_t i = 0; i < vertex_count; i++) {
        wasm_free(qem_vertices[i].adjacent_vertices);
    }
    wasm_free(qem_vertices);
    
    // Return successful result
    result.vertices = new_vertices;
    result.indices = new_indices;
    result.vertex_count = new_vertex_index;
    result.index_count = new_index_count * 3;
    result.success = 1;
    
    return result;
}

// Free mesh decimation result
void free_mesh_decimate_result(MeshDecimateResult* result) {
    if (result && result->success) {
        wasm_free(result->vertices);
        wasm_free(result->indices);
        result->vertices = NULL;
        result->indices = NULL;
        result->vertex_count = 0;
        result->index_count = 0;
        result->success = 0;
    }
}
