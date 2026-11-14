#!/usr/bin/env node
/**
 * Verifies Communicator's charcoal/cream colour pairs against WCAG AA using
 * the shared @nadeemmurtaza/design-system contrast engine. Run via
 * `npm run check-contrast`.
 */
import { contrastRatioHex, wcagLevel } from '@nadeemmurtaza/design-system';

const pairs = [
  ['white button label on charcoal fill', '#ffffff', '#1c1917', 4.5],
  ['ink text on cream page', '#1c1917', '#faf8f3', 4.5],
  ['muted ink text on cream page', '#57534e', '#faf8f3', 4.5],
  ['faint ink on cream page (non-text, 3:1 floor)', '#908b88', '#faf8f3', 3],
  ['border on cream page (non-text, 3:1 floor)', '#928f88', '#faf8f3', 3],
];

let failed = false;
for (const [label, fg, bg, floor] of pairs) {
  const ratio = contrastRatioHex(fg, bg);
  const level = wcagLevel(ratio);
  const ok = ratio >= floor;
  if (!ok) failed = true;
  console.log(`${ok ? 'OK  ' : 'FAIL'} ${label}: ${ratio.toFixed(2)}:1 (${level}, floor ${floor}:1)`);
}

if (failed) {
  console.error('\nSome pairs are under their WCAG floor.');
  process.exit(1);
}
console.log('\nAll checked pairs clear their WCAG floor.');
