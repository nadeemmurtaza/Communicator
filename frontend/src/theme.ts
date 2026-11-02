/**
 * Communicator's own brand VALUES, typed against @nadeemmurtaza/design-system's
 * BrandTheme contract -- the ecosystem-shared engine. index.css's :root custom
 * properties are the actual rendered values (unchanged by this file); this is
 * the typed source of truth they're transcribed from, so a future CSS build
 * step or another surface (e.g. a React Native client) can derive the same
 * values without re-deriving by eye. See packages/design-system/README.md in
 * core-newax for the engine/values split.
 */
import type { BrandTheme } from '@nadeemmurtaza/design-system';

export const THEME: BrandTheme = {
  colors: {
    neutral: {
      50: '#faf8f3',
      100: '#f5f3ef',
      // 200/300 darkened from #e7e2d6/#a8a29e -- same hue, lightness only --
      // until each cleared the WCAG 3:1 non-text floor against the cream
      // page (#faf8f3): #e7e2d6 was 1.22:1, #a8a29e was 2.38:1. Computed via
      // @nadeemmurtaza/design-system's darkenToContrastHex; see
      // scripts/check-contrast.mjs.
      200: '#928f88',
      300: '#908b88',
      400: '#57534e',
      500: '#1c1917',
    },
    accent: { DEFAULT: '#1c1917', hover: '#0c0a09' },
    white: '#ffffff',
    surface: '#ffffff',
    overlay: '#ffffff',
    inverse: '#1c1917',
    onInverse: '#ffffff',
    scrim: '#1c1917',
    star: '#f5b841',
    feedback: { positive: '#0f7a4d', negative: '#b42318' },
    chart: ['#1c1917'],
  },
  colorsDark: {
    neutral: {
      50: '#1c1917',
      100: '#292524',
      200: '#44403c',
      300: '#a8a29e',
      400: '#d6d3d1',
      500: '#faf8f3',
    },
    accent: { DEFAULT: '#faf8f3', hover: '#f5f3ef' },
    accentInk: { DEFAULT: '#d6d3d1', hover: '#e7e2d6' },
    white: '#ffffff',
    surface: '#292524',
    overlay: '#292524',
    inverse: '#faf8f3',
    onInverse: '#1c1917',
    scrim: '#000000',
    star: '#f5b841',
    feedback: { positive: '#3dcb8b', negative: '#ff8a80' },
    chart: ['#d6d3d1'],
  },
  radii: { sharp: 8, lg: 16, pill: 9999 },
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
