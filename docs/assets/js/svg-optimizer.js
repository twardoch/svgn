/**
 * SVG Optimizer Tool
 * Frontend interface for SVGN WebAssembly optimizer
 */

class SVGOptimizer {
  constructor() {
    this.files = [];
    this.wasmModule = null;
    this.isLoading = false;
    
    this.init();
  }

  async init() {
    this.setupEventListeners();
    this.updateUI();
    
    // Load WASM module when available
    try {
      // Placeholder for WASM loading - will be implemented in Thread F
      console.log('WASM module loading will be implemented in Thread F');
    } catch (error) {
      console.warn('WASM module not available:', error);
      this.showFallbackMessage();
    }
  }

  setupEventListeners() {
    // File drop zone
    const dropZone = document.getElementById('file-drop-zone');
    const fileInput = document.getElementById('file-input');
    const fileSelectBtn = document.getElementById('file-select-btn');

    if (dropZone) {
      dropZone.addEventListener('dragover', this.handleDragOver.bind(this));
      dropZone.addEventListener('dragleave', this.handleDragLeave.bind(this));
      dropZone.addEventListener('drop', this.handleDrop.bind(this));
      dropZone.addEventListener('click', () => fileInput?.click());
    }

    if (fileInput) {
      fileInput.addEventListener('change', this.handleFileSelect.bind(this));
    }

    if (fileSelectBtn) {
      fileSelectBtn.addEventListener('click', () => fileInput?.click());
    }

    // Configuration changes
    const presetSelect = document.getElementById('preset-select');
    if (presetSelect) {
      presetSelect.addEventListener('change', this.handlePresetChange.bind(this));
    }

    // Plugin toggles
    const pluginToggles = document.querySelectorAll('.plugin-toggle input[type="checkbox"]');
    pluginToggles.forEach(toggle => {
      toggle.addEventListener('change', this.handlePluginToggle.bind(this));
    });

    // Copy buttons
    const copyOriginal = document.getElementById('copy-original');
    const copyOptimized = document.getElementById('copy-optimized');
    
    if (copyOriginal) {
      copyOriginal.addEventListener('click', () => this.copyToClipboard('original'));
    }
    
    if (copyOptimized) {
      copyOptimized.addEventListener('click', () => this.copyToClipboard('optimized'));
    }

    // Download buttons
    const downloadOptimized = document.getElementById('download-optimized');
    const downloadAll = document.getElementById('download-all');
    
    if (downloadOptimized) {
      downloadOptimized.addEventListener('click', this.downloadOptimized.bind(this));
    }
    
    if (downloadAll) {
      downloadAll.addEventListener('click', this.downloadAll.bind(this));
    }
  }

  handleDragOver(e) {
    e.preventDefault();
    e.dataTransfer.dropEffect = 'copy';
    e.currentTarget.classList.add('dragover');
  }

  handleDragLeave(e) {
    e.preventDefault();
    e.currentTarget.classList.remove('dragover');
  }

  handleDrop(e) {
    e.preventDefault();
    e.currentTarget.classList.remove('dragover');
    
    const files = Array.from(e.dataTransfer.files).filter(file => 
      file.type === 'image/svg+xml' || file.name.endsWith('.svg')
    );
    
    if (files.length > 0) {
      this.processFiles(files);
    } else {
      this.showError('Please drop SVG files only.');
    }
  }

  handleFileSelect(e) {
    const files = Array.from(e.target.files);
    if (files.length > 0) {
      this.processFiles(files);
    }
  }

  async processFiles(files) {
    this.files = files;
    this.showProcessing();
    
    try {
      // For now, simulate processing - real WASM integration in Thread F
      await this.simulateOptimization(files[0]);
      this.showResults();
    } catch (error) {
      this.showError(`Error processing files: ${error.message}`);
    }
  }

  async simulateOptimization(file) {
    const content = await this.readFileAsText(file);
    
    // Simulate basic optimization
    const optimized = content
      .replace(/<!--[\s\S]*?-->/g, '') // Remove comments
      .replace(/\s+/g, ' ') // Normalize whitespace
      .replace(/>\s+</g, '><') // Remove whitespace between tags
      .trim();

    // Update previews
    this.updatePreview('original', content);
    this.updatePreview('optimized', optimized);
    
    // Update statistics
    this.updateStatistics(content.length, optimized.length);
    
    // Store for download
    this.originalContent = content;
    this.optimizedContent = optimized;
    this.currentFileName = file.name;
  }

  readFileAsText(file) {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = e => resolve(e.target.result);
      reader.onerror = e => reject(new Error('Failed to read file'));
      reader.readAsText(file);
    });
  }

  updatePreview(type, content) {
    const container = document.getElementById(`${type}-preview`);
    const codeDisplay = document.getElementById(`${type}-code`);
    
    if (container) {
      container.innerHTML = content;
    }
    
    if (codeDisplay) {
      codeDisplay.innerHTML = this.escapeHtml(content);
    }
  }

  updateStatistics(originalSize, optimizedSize) {
    const reduction = originalSize - optimizedSize;
    const ratio = ((reduction / originalSize) * 100).toFixed(1);
    
    this.updateElement('original-size', this.formatBytes(originalSize));
    this.updateElement('optimized-size', this.formatBytes(optimizedSize));
    this.updateElement('size-reduction', this.formatBytes(reduction));
    this.updateElement('compression-ratio', `${ratio}%`);
    
    this.updateElement('original-size-display', this.formatBytes(originalSize));
    this.updateElement('optimized-size-display', this.formatBytes(optimizedSize));
  }

  updateElement(id, content) {
    const element = document.getElementById(id);
    if (element) {
      element.textContent = content;
    }
  }

  formatBytes(bytes) {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  }

  escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
  }

  async copyToClipboard(type) {
    const content = type === 'original' ? this.originalContent : this.optimizedContent;
    
    try {
      await navigator.clipboard.writeText(content);
      this.showToast(`${type} SVG copied to clipboard!`);
    } catch (error) {
      console.error('Failed to copy to clipboard:', error);
      this.showToast('Failed to copy to clipboard', 'error');
    }
  }

  downloadOptimized() {
    if (!this.optimizedContent) return;
    
    const blob = new Blob([this.optimizedContent], { type: 'image/svg+xml' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    
    a.href = url;
    a.download = this.currentFileName.replace('.svg', '_optimized.svg');
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }

  downloadAll() {
    // Placeholder for batch download - will be implemented when multiple files are supported
    this.showToast('Batch download will be available with multiple file support');
  }

  handlePresetChange(e) {
    const preset = e.target.value;
    console.log('Preset changed to:', preset);
    // Will update plugin configurations based on preset
  }

  handlePluginToggle(e) {
    const plugin = e.target.closest('.plugin-toggle');
    const pluginName = plugin?.querySelector('span')?.textContent;
    console.log('Plugin toggled:', pluginName, e.target.checked);
    // Will update plugin configuration
  }

  showProcessing() {
    this.hideAllSections();
    this.showElement('processing-status');
  }

  showResults() {
    this.hideAllSections();
    this.showElement('results-area');
  }

  showError(message) {
    this.hideAllSections();
    this.updateElement('error-message', message);
    this.showElement('error-display');
  }

  showFallbackMessage() {
    const dropZone = document.getElementById('file-drop-zone');
    if (dropZone) {
      dropZone.innerHTML = `
        <div class="space-y-4">
          <svg class="w-16 h-16 mx-auto text-warning" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z"></path>
          </svg>
          <div>
            <h3 class="text-lg font-semibold">WASM Module Not Available</h3>
            <p class="text-sm text-base-content/70">The WebAssembly optimizer is not yet loaded.</p>
            <p class="text-xs text-base-content/50 mt-2">This will be available once Thread F (WASM API) is implemented.</p>
          </div>
        </div>
      `;
    }
  }

  hideAllSections() {
    ['processing-status', 'results-area', 'error-display'].forEach(id => {
      this.hideElement(id);
    });
  }

  showElement(id) {
    const element = document.getElementById(id);
    if (element) {
      element.style.display = 'block';
    }
  }

  hideElement(id) {
    const element = document.getElementById(id);
    if (element) {
      element.style.display = 'none';
    }
  }

  showToast(message, type = 'success') {
    // Simple toast implementation
    const toast = document.createElement('div');
    toast.className = `alert alert-${type} fixed top-4 right-4 w-auto z-50 fade-in`;
    toast.innerHTML = `
      <span>${message}</span>
      <button class="btn btn-sm btn-circle btn-ghost ml-2" onclick="this.parentElement.remove()">
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
        </svg>
      </button>
    `;
    
    document.body.appendChild(toast);
    
    setTimeout(() => {
      if (toast.parentElement) {
        toast.remove();
      }
    }, 5000);
  }

  updateUI() {
    // Update UI state based on current state
    this.hideAllSections();
  }
}

// Initialize when DOM is ready
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', () => {
    window.svgOptimizer = new SVGOptimizer();
  });
} else {
  window.svgOptimizer = new SVGOptimizer();
}