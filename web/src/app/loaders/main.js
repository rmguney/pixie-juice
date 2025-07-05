// Import WASM module
import init, { 
    ImageOptimizer, 
    MeshOptimizer,
    WasmOptConfig,
    FileHandler 
} from './pkg/pixie_juice_web.js';

// App state
let wasmModule;
let imageOptimizer;
let meshOptimizer;
let selectedFiles = [];
let processedResults = [];

// DOM elements
const elements = {
    loading: document.getElementById('loading'),
    app: document.getElementById('app'),
    dropZone: document.getElementById('drop-zone'),
    fileInput: document.getElementById('file-input'),
    selectFilesBtn: document.getElementById('select-files'),
    settings: document.getElementById('settings'),
    qualitySlider: document.getElementById('quality'),
    qualityValue: document.getElementById('quality-value'),
    formatSelect: document.getElementById('format'),
    filesContainer: document.getElementById('files-container'),
    filesList: document.getElementById('files-list'),
    processAllBtn: document.getElementById('process-all'),
    processing: document.getElementById('processing'),
    progressFill: document.getElementById('progress-fill'),
    progressText: document.getElementById('progress-text'),
    results: document.getElementById('results'),
    resultsList: document.getElementById('results-list'),
    downloadAllBtn: document.getElementById('download-all'),
    resetBtn: document.getElementById('reset'),
    toast: document.getElementById('toast'),
    toastMessage: document.getElementById('toast-message'),
    toastClose: document.getElementById('toast-close')
};

// Initialize app
async function initializeApp() {
    try {
        console.log('Initializing WASM module...');
        wasmModule = await init();
        imageOptimizer = new ImageOptimizer();
        meshOptimizer = new MeshOptimizer();
        
        elements.loading.classList.add('hidden');
        elements.app.classList.remove('hidden');
        
        setupEventListeners();
        showToast('Pixie Juice ready! Drop images or 3D models to get started.', 'success');
        
    } catch (error) {
        console.error('Failed to initialize WASM:', error);
        showToast('Failed to load. Please refresh the page.', 'error');
    }
}

// Event listeners
function setupEventListeners() {
    // File selection
    elements.selectFilesBtn.addEventListener('click', () => {
        elements.fileInput.click();
    });
    
    elements.fileInput.addEventListener('change', handleFileSelect);
    
    // Drag and drop
    setupDragAndDrop();
    
    // Settings
    elements.qualitySlider.addEventListener('input', () => {
        elements.qualityValue.textContent = elements.qualitySlider.value;
    });
    
    // Process and download
    elements.processAllBtn.addEventListener('click', processAllFiles);
    elements.downloadAllBtn.addEventListener('click', downloadAllResults);
    elements.resetBtn.addEventListener('click', resetApp);
    
    // Toast
    elements.toastClose.addEventListener('click', hideToast);
}

// Drag and drop setup
function setupDragAndDrop() {
    ['dragenter', 'dragover', 'dragleave', 'drop'].forEach(eventName => {
        elements.dropZone.addEventListener(eventName, preventDefaults, false);
    });
    
    ['dragenter', 'dragover'].forEach(eventName => {
        elements.dropZone.addEventListener(eventName, () => {
            elements.dropZone.classList.add('drag-over');
        }, false);
    });
    
    ['dragleave', 'drop'].forEach(eventName => {
        elements.dropZone.addEventListener(eventName, () => {
            elements.dropZone.classList.remove('drag-over');
        }, false);
    });
    
    elements.dropZone.addEventListener('drop', (e) => {
        const files = Array.from(e.dataTransfer.files);
        handleFiles(files);
    }, false);
}

function preventDefaults(e) {
    e.preventDefault();
    e.stopPropagation();
}

// File handling
function handleFileSelect(event) {
    const files = Array.from(event.target.files);
    handleFiles(files);
}

function handleFiles(files) {
    // Filter supported files
    const supportedFiles = files.filter(file => {
        return file.type.startsWith('image/') || 
               file.name.toLowerCase().endsWith('.obj') ||
               file.name.toLowerCase().endsWith('.ply') ||
               file.name.toLowerCase().endsWith('.stl');
    });
    
    if (supportedFiles.length === 0) {
        showToast('Please select valid image or 3D model files.', 'error');
        return;
    }
    
    if (supportedFiles.length !== files.length) {
        showToast(`${files.length - supportedFiles.length} unsupported files were ignored.`, 'warning');
    }
    
    // Add to selected files (avoid duplicates by name)
    supportedFiles.forEach(file => {
        if (!selectedFiles.some(f => f.name === file.name)) {
            selectedFiles.push(file);
        }
    });
    
    updateFilesDisplay();
    
    const imageCount = supportedFiles.filter(f => f.type.startsWith('image/')).length;
    const meshCount = supportedFiles.length - imageCount;
    let message = '';
    if (imageCount > 0 && meshCount > 0) {
        message = `${imageCount} image(s) and ${meshCount} 3D model(s) added.`;
    } else if (imageCount > 0) {
        message = `${imageCount} image(s) added.`;
    } else {
        message = `${meshCount} 3D model(s) added.`;
    }
    showToast(message, 'success');
}

// Update files display
function updateFilesDisplay() {
    if (selectedFiles.length === 0) {
        elements.filesContainer.classList.add('hidden');
        return;
    }
    
    elements.filesContainer.classList.remove('hidden');
    elements.filesList.innerHTML = '';
    
    selectedFiles.forEach((file, index) => {
        const fileItem = document.createElement('div');
        fileItem.className = 'file-item';
        fileItem.innerHTML = `
            <div class="file-info">
                <div class="file-name">${file.name}</div>
                <div class="file-meta">
                    <span>${formatFileSize(file.size)}</span>
                    <span>${file.type.split('/')[1].toUpperCase()}</span>
                </div>
            </div>
            <button class="btn small secondary" onclick="removeFile(${index})">Remove</button>
        `;
        elements.filesList.appendChild(fileItem);
    });
}

// Remove file
window.removeFile = function(index) {
    selectedFiles.splice(index, 1);
    updateFilesDisplay();
};

// Process all files
async function processAllFiles() {
    if (selectedFiles.length === 0) {
        showToast('Please select some files first.', 'error');
        return;
    }
    
    // Show processing UI
    elements.filesContainer.classList.add('hidden');
    elements.processing.classList.remove('hidden');
    elements.results.classList.add('hidden');
    
    processedResults = [];
    const quality = parseInt(elements.qualitySlider.value);
    const outputFormat = elements.formatSelect.value;
    
    try {
        for (let i = 0; i < selectedFiles.length; i++) {
            const file = selectedFiles[i];
            const progress = ((i + 1) / selectedFiles.length) * 100;
            
            // Update progress
            elements.progressFill.style.width = `${progress}%`;
            elements.progressText.textContent = `Processing ${file.name}... (${i + 1}/${selectedFiles.length})`;
            
            try {
                // Read file
                const fileData = await FileHandler.read_file_as_bytes(file);
                
                // Create config
                const config = new WasmOptConfig();
                config.set_quality(quality);
                config.set_preserve_metadata(true);
                
                let optimizedData;
                let targetFormat;
                
                // Check if it's an image or mesh file
                if (file.type.startsWith('image/')) {
                    // Image processing
                    const inputFormat = file.type.split('/')[1];
                    targetFormat = outputFormat === 'auto' ? inputFormat : outputFormat;
                    
                    optimizedData = await imageOptimizer.process_image_file(
                        fileData, 
                        inputFormat, 
                        targetFormat,
                        config
                    );
                } else {
                    // Mesh processing  
                    const extension = file.name.toLowerCase().split('.').pop();
                    targetFormat = extension; // Keep original format for meshes
                    
                    optimizedData = await meshOptimizer.process_mesh_file(
                        fileData, 
                        extension,
                        config
                    );
                }
                
                // Calculate savings
                const originalSize = fileData.length;
                const optimizedSize = optimizedData.length;
                const savings = ((originalSize - optimizedSize) / originalSize) * 100;
                
                processedResults.push({
                    originalFile: file,
                    originalData: fileData,
                    optimizedData: optimizedData,
                    targetFormat: targetFormat,
                    originalSize: originalSize,
                    optimizedSize: optimizedSize,
                    savings: savings,
                    success: true,
                    fileType: file.type.startsWith('image/') ? 'image' : 'mesh'
                });
                
            } catch (error) {
                console.error(`Error processing ${file.name}:`, error);
                processedResults.push({
                    originalFile: file,
                    error: error.toString(),
                    success: false,
                    fileType: file.type.startsWith('image/') ? 'image' : 'mesh'
                });
            }
        }
        
        // Show results
        showResults();
        showToast('All files processed successfully!', 'success');
        
    } catch (error) {
        console.error('Processing error:', error);
        showToast('Processing failed. Please try again.', 'error');
    }
}

// Show results
function showResults() {
    elements.processing.classList.add('hidden');
    elements.results.classList.remove('hidden');
    elements.resultsList.innerHTML = '';
    
    let totalOriginalSize = 0;
    let totalOptimizedSize = 0;
    let successCount = 0;
    
    processedResults.forEach((result, index) => {
        const resultItem = document.createElement('div');
        resultItem.className = 'result-item';
        
        if (result.success) {
            totalOriginalSize += result.originalSize;
            totalOptimizedSize += result.optimizedSize;
            successCount++;
            
            const savingsClass = result.savings > 0 ? 'savings-positive' : 'savings-negative';
            const savingsText = result.savings > 0 ? `-${result.savings.toFixed(1)}%` : `+${Math.abs(result.savings).toFixed(1)}%`;
            
            resultItem.innerHTML = `
                <div class="result-info">
                    <div class="file-name">${result.originalFile.name}</div>
                    <div class="result-savings">
                        <span>${formatFileSize(result.originalSize)} → ${formatFileSize(result.optimizedSize)}</span>
                        <span class="${savingsClass}">${savingsText}</span>
                    </div>
                </div>
                <button class="btn small primary" onclick="downloadSingle(${index})">Download</button>
            `;
        } else {
            resultItem.innerHTML = `
                <div class="result-info">
                    <div class="file-name">${result.originalFile.name}</div>
                    <div class="result-savings">
                        <span style="color: var(--error)">Failed: ${result.error}</span>
                    </div>
                </div>
            `;
        }
        
        elements.resultsList.appendChild(resultItem);
    });
    
    // Update header with summary
    if (successCount > 0) {
        const totalSavings = ((totalOriginalSize - totalOptimizedSize) / totalOriginalSize) * 100;
        document.querySelector('.results-header h3').textContent = 
            `${successCount} files optimized (${totalSavings.toFixed(1)}% total savings)`;
    }
}

// Download functions
window.downloadSingle = function(index) {
    const result = processedResults[index];
    if (!result.success) return;
    
    // Determine MIME type based on file type
    const mimeType = result.fileType === 'image' 
        ? `image/${result.targetFormat}` 
        : 'application/octet-stream';
    
    const blob = new Blob([result.optimizedData], { type: mimeType });
    const url = URL.createObjectURL(blob);
    
    const a = document.createElement('a');
    a.href = url;
    
    // Generate appropriate filename
    const extension = result.fileType === 'image' ? result.targetFormat : result.targetFormat;
    const baseName = result.originalFile.name.replace(/\.[^/.]+$/, '');
    a.download = `optimized_${baseName}.${extension}`;
    
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    
    URL.revokeObjectURL(url);
    showToast('Download started!', 'success');
};

function downloadAllResults() {
    const successfulResults = processedResults.filter(r => r.success);
    
    if (successfulResults.length === 0) {
        showToast('No files to download.', 'warning');
        return;
    }
    
    // Download each file with a small delay
    successfulResults.forEach((result, index) => {
        setTimeout(() => {
            window.downloadSingle(processedResults.indexOf(result));
        }, index * 200);
    });
    
    showToast(`Downloading ${successfulResults.length} files...`, 'success');
}

// Reset app
function resetApp() {
    selectedFiles = [];
    processedResults = [];
    
    elements.filesContainer.classList.add('hidden');
    elements.processing.classList.add('hidden');
    elements.results.classList.add('hidden');
    
    elements.fileInput.value = '';
    
    showToast('Ready for new files!', 'success');
}

// Toast notifications
function showToast(message, type = 'info') {
    elements.toastMessage.textContent = message;
    elements.toast.className = `toast ${type}`;
    elements.toast.classList.remove('hidden');
    
    setTimeout(hideToast, 5000);
}

function hideToast() {
    elements.toast.classList.add('hidden');
}

// Utility functions
function formatFileSize(bytes) {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
}

// Initialize app
initializeApp();
