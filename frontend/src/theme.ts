/**
 * Communicator design tokens -- Intercom-inspired palette.
 *
 * Warm cream canvas (#faf9f6), off-black ink (#111111), Fin Orange accent
 * (#ff5600), sharp 4px geometry. index.css :root custom properties are the
 * rendered values; this file is the typed source of truth for programmatic
 * consumers (charts, dynamic theming).
 */
export const THEME = {
  colors: {
    neutral: {
      50: '#faf9f6',
      100: '#f1eee9',
      200: '#dedbd6',
      300: '#928f88',
      400: '#57534e',
      500: '#111111',
    },
    accent: { DEFAULT: '#ff5600', hover: '#e64e00' },
    white: '#ffffff',
    surface: '#ffffff',
    overlay: '#ffffff',
    inverse: '#111111',
    onInverse: '#ffffff',
    scrim: '#000000',
    star: '#f5b841',
    feedback: { positive: '#0bdf50', negative: '#c41c1c' },
    chart: ['#ff5600', '#65b5ff', '#0bdf50', '#ff2067', '#b3e01c'],
  },
  colorsDark: {
    neutral: {
      50: '#111111',
      100: '#1a1a1a',
      200: '#333333',
      300: '#928f88',
      400: '#dedbd6',
      500: '#faf9f6',
    },
    accent: { DEFAULT: '#ff6a1a', hover: '#ff7b2e' },
    accentInk: { DEFAULT: '#dedbd6', hover: '#f1eee9' },
    white: '#ffffff',
    surface: '#1a1a1a',
    overlay: '#1a1a1a',
    inverse: '#faf9f6',
    onInverse: '#111111',
    scrim: '#000000',
    star: '#f5b841',
    feedback: { positive: '#3dcb8b', negative: '#ff8a80' },
    chart: ['#ff6a1a', '#65b5ff', '#3dcb8b', '#ff2067', '#b3e01c'],
  },
  radii: { sharp: 4, lg: 8, pill: 9999 },
  type: {
    micro: { size: 11, line: 16 },
    tiny: { size: 12, line: 16 },
    small: { size: 14, line: 20 },
    body: { size: 16, line: 24 },
    large: { size: 18, line: 28 },
    heading: { size: 20, line: 28 },
    title: { size: 24, line: 32 },
    display: { size: 30, line: 36 },
    metric: { size: 40, line: 44 },
  },
  fonts: {
    googleFont: 'Inter',
    displayStack: ['Inter', 'ui-sans-serif', 'system-ui', '-apple-system', 'Segoe UI', 'Roboto', 'sans-serif'],
    sansStack: ['Inter', 'ui-sans-serif', 'system-ui', '-apple-system', 'Segoe UI', 'Roboto', 'sans-serif'],
  },
  space: { 0: 0, 1: 2, 2: 4, 3: 8, 4: 12, 5: 16, 6: 24, 7: 32, 8: 48, 9: 64 },
  size: {
    control: { sm: 32, md: 40, lg: 48, xl: 56 },
    icon: { sm: 16, md: 20, lg: 24 },
    touch: { min: 44, pos: 48 },
  },
  elevation: {
    light: {
      flat: 'none',
      raised: '0 1px 2px 0 rgb(16 24 40 / 0.04), 0 1px 3px 0 rgb(16 24 40 / 0.06)',
      overlay: '0 4px 8px -2px rgb(16 24 40 / 0.06), 0 12px 24px -6px rgb(16 24 40 / 0.1)',
      modal: '0 8px 16px -4px rgb(16 24 40 / 0.1), 0 24px 56px -12px rgb(16 24 40 / 0.22)',
    },
    dark: {
      flat: 'none',
      raised: '0 1px 2px 0 rgb(0 0 0 / 0.5), 0 1px 3px 0 rgb(0 0 0 / 0.4)',
      overlay: '0 4px 8px -2px rgb(0 0 0 / 0.5), 0 12px 24px -6px rgb(0 0 0 / 0.55)',
      modal: '0 8px 16px -4px rgb(0 0 0 / 0.55), 0 24px 56px -12px rgb(0 0 0 / 0.7)',
    },
  },
  layers: { base: 0, sticky: 10, dropdown: 20, overlay: 30, modal: 40, toast: 50 },
  motion: {
    duration: { fast: 140, normal: 220, slow: 320 },
    easing: { standard: [0.2, 0, 0, 1], decelerate: [0.22, 0.61, 0.36, 1], settle: [0.16, 1, 0.3, 1] },
    distance: {},
    scale: {},
    stagger: { step: 0, fine: 0 },
    speed: { read: 0 },
  },
};
