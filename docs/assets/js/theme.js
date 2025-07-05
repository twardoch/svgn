/**
 * Theme Management for SVGN Documentation
 * Handles dark/light theme switching with system preference detection
 */

class ThemeManager {
  constructor() {
    this.STORAGE_KEY = 'svgn-theme';
    this.THEMES = {
      LIGHT: 'light',
      DARK: 'dark',
      SYSTEM: 'system'
    };
    
    this.init();
  }

  init() {
    // Initialize theme on page load
    this.setTheme(this.getStoredTheme());
    
    // Listen for system theme changes
    if (window.matchMedia) {
      window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', (e) => {
        if (this.getStoredTheme() === this.THEMES.SYSTEM) {
          this.applyTheme(e.matches ? this.THEMES.DARK : this.THEMES.LIGHT);
        }
      });
    }
    
    // Update theme toggle button state
    this.updateThemeToggleButton();
  }

  getStoredTheme() {
    return localStorage.getItem(this.STORAGE_KEY) || this.THEMES.SYSTEM;
  }

  getSystemTheme() {
    if (window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches) {
      return this.THEMES.DARK;
    }
    return this.THEMES.LIGHT;
  }

  getCurrentTheme() {
    const stored = this.getStoredTheme();
    return stored === this.THEMES.SYSTEM ? this.getSystemTheme() : stored;
  }

  setTheme(theme) {
    localStorage.setItem(this.STORAGE_KEY, theme);
    
    const actualTheme = theme === this.THEMES.SYSTEM ? this.getSystemTheme() : theme;
    this.applyTheme(actualTheme);
    this.updateThemeToggleButton();
  }

  applyTheme(theme) {
    document.documentElement.setAttribute('data-theme', theme);
    
    // Also set class for compatibility
    if (theme === this.THEMES.DARK) {
      document.documentElement.classList.add('dark');
    } else {
      document.documentElement.classList.remove('dark');
    }
    
    // Dispatch custom event for other components
    window.dispatchEvent(new CustomEvent('theme-changed', { 
      detail: { theme, isSystem: this.getStoredTheme() === this.THEMES.SYSTEM }
    }));
  }

  toggleTheme() {
    const currentStored = this.getStoredTheme();
    const currentActual = this.getCurrentTheme();
    
    // Cycle through: light -> dark -> system
    let newTheme;
    if (currentStored === this.THEMES.LIGHT) {
      newTheme = this.THEMES.DARK;
    } else if (currentStored === this.THEMES.DARK) {
      newTheme = this.THEMES.SYSTEM;
    } else {
      newTheme = this.THEMES.LIGHT;
    }
    
    this.setTheme(newTheme);
  }

  updateThemeToggleButton() {
    const buttons = document.querySelectorAll('.theme-toggle');
    const currentTheme = this.getCurrentTheme();
    const storedTheme = this.getStoredTheme();
    
    buttons.forEach(button => {
      // Update button icon
      const lightIcon = button.querySelector('.theme-icon-light');
      const darkIcon = button.querySelector('.theme-icon-dark');
      const systemIcon = button.querySelector('.theme-icon-system');
      
      // Hide all icons first
      if (lightIcon) lightIcon.style.display = 'none';
      if (darkIcon) darkIcon.style.display = 'none';
      if (systemIcon) systemIcon.style.display = 'none';
      
      // Show appropriate icon based on stored preference
      if (storedTheme === this.THEMES.SYSTEM && systemIcon) {
        systemIcon.style.display = 'block';
      } else if (currentTheme === this.THEMES.DARK && darkIcon) {
        darkIcon.style.display = 'block';
      } else if (lightIcon) {
        lightIcon.style.display = 'block';
      }
      
      // Update button title
      let title = 'Toggle theme';
      if (storedTheme === this.THEMES.SYSTEM) {
        title = `Auto (${currentTheme}) - Click for light`;
      } else if (currentTheme === this.THEMES.DARK) {
        title = 'Dark theme - Click for auto';
      } else {
        title = 'Light theme - Click for dark';
      }
      button.setAttribute('title', title);
    });
  }
}

// Global theme management function for backward compatibility
function toggleTheme() {
  window.themeManager.toggleTheme();
}

// Initialize theme manager when DOM is ready
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', () => {
    window.themeManager = new ThemeManager();
  });
} else {
  window.themeManager = new ThemeManager();
}