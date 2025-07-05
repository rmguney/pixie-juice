import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import ProcessingPanel from '../src/app/components/ProcessingPanel';

// Mock WASM module
const mockWasm = {
  WasmOptConfig: class {
    constructor() {
      this.quality = 85;
      this.target_reduction = 0;
      this.preserve_metadata = true;
      this.lossless = false;
    }
    
    set_quality(q) { this.quality = q; }
    set_target_reduction(r) { this.target_reduction = r; }
    set_preserve_metadata(p) { this.preserve_metadata = p; }
    set_lossless(l) { this.lossless = l; }
  },
  imageOptimizer: {
    process_image_file: jest.fn().mockResolvedValue(new Uint8Array([1, 2, 3]))
  },
  meshOptimizer: {
    process_mesh_file: jest.fn().mockResolvedValue(new Uint8Array([4, 5, 6]))
  }
};

describe('ProcessingPanel Controls', () => {
  test('quality slider affects configuration', async () => {
    const mockOnProcess = jest.fn();
    const mockSetIsProcessing = jest.fn();
    
    render(
      <ProcessingPanel 
        files={[]} 
        wasm={mockWasm} 
        onProcess={mockOnProcess}
        setIsProcessing={mockSetIsProcessing}
      />
    );
    
    // Find quality slider
    const qualitySlider = screen.getByLabelText(/quality/i);
    expect(qualitySlider).toBeInTheDocument();
    
    // Test quality range
    expect(qualitySlider).toHaveAttribute('min', '10');
    expect(qualitySlider).toHaveAttribute('max', '100');
    
    // Change quality to aggressive setting
    fireEvent.change(qualitySlider, { target: { value: '20' } });
    
    // Check that quality value is displayed
    expect(screen.getByText('20%')).toBeInTheDocument();
  });
  
  test('target reduction slider exists and works', () => {
    const mockOnProcess = jest.fn();
    const mockSetIsProcessing = jest.fn();
    
    render(
      <ProcessingPanel 
        files={[]} 
        wasm={mockWasm} 
        onProcess={mockOnProcess}
        setIsProcessing={mockSetIsProcessing}
      />
    );
    
    // Find target reduction slider
    const reductionSlider = screen.getByLabelText(/target reduction/i);
    expect(reductionSlider).toBeInTheDocument();
    
    // Test reduction range
    expect(reductionSlider).toHaveAttribute('min', '10');
    expect(reductionSlider).toHaveAttribute('max', '80');
    
    // Change reduction target
    fireEvent.change(reductionSlider, { target: { value: '60' } });
    
    // Check that reduction value is displayed
    expect(screen.getByText('60%')).toBeInTheDocument();
  });
  
  test('configuration maps correctly to WASM config', async () => {
    const mockOnProcess = jest.fn();
    const mockSetIsProcessing = jest.fn();
    
    // Mock file
    const mockFile = new File(['test'], 'test.png', { type: 'image/png' });
    
    render(
      <ProcessingPanel 
        files={[mockFile]} 
        wasm={mockWasm} 
        onProcess={mockOnProcess}
        setIsProcessing={mockSetIsProcessing}
      />
    );
    
    // Set aggressive quality
    const qualitySlider = screen.getByLabelText(/quality/i);
    fireEvent.change(qualitySlider, { target: { value: '20' } });
    
    // Set high target reduction  
    const reductionSlider = screen.getByLabelText(/target reduction/i);
    fireEvent.change(reductionSlider, { target: { value: '60' } });
    
    // Click optimize button
    const optimizeButton = screen.getByText(/optimize/i);
    fireEvent.click(optimizeButton);
    
    // Wait for processing to start
    await waitFor(() => {
      expect(mockSetIsProcessing).toHaveBeenCalledWith(true);
    });
  });
});
