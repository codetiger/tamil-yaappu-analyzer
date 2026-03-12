/**
 * junction-view.js — Renders thalai (junction) data between words.
 */
import { titleCase, ASAI_NAMES } from '../data-mapper.js';

/**
 * Render junction indicators between word blocks in the verse display.
 */
export function renderJunctions(verseZone, paa) {
  if (!paa?.thalaikal?.length) return;

  verseZone.querySelectorAll('.junction-indicator').forEach(el => el.remove());

  const wordBlocks = verseZone.querySelectorAll('.word-block');
  if (wordBlocks.length === 0) return;

  paa.thalaikal.forEach(thalai => {
    const fromBlock = wordBlocks[thalai.from_sol_index];
    if (!fromBlock) return;

    const sameAsai = thalai.eerru_asai === thalai.muthal_asai;
    const sameSeerCat = thalai.from_seer_category === thalai.to_seer_category;
    const isIntraCompound = thalai.is_intra_compound;

    let status = 'valid';
    if (isIntraCompound) status = 'compound';
    else if (thalai.is_cross_adi) status = 'crossline';
    else if (sameAsai && sameSeerCat) status = 'warning';

    const indicator = document.createElement('div');
    indicator.className = `junction-indicator junction-${status}`;

    const eerruTamil = ASAI_NAMES[thalai.eerru_asai] || '';
    const muthalTamil = ASAI_NAMES[thalai.muthal_asai] || '';
    const asaiInfo = `${titleCase(thalai.eerru_asai)} (${eerruTamil}) → ${titleCase(thalai.muthal_asai)} (${muthalTamil})`;
    indicator.textContent = asaiInfo;

    if (isIntraCompound) {
      indicator.title = 'Intra-compound junction (filtered)';
    } else if (thalai.is_cross_adi) {
      indicator.title = `Cross-line junction: ${asaiInfo}`;
    } else if (status === 'warning') {
      indicator.title = `Thalai break: same asai type`;
    } else {
      indicator.title = `Valid alternation`;
    }

    fromBlock.appendChild(indicator);
  });
}

/**
 * Render junction details in the detail panel for a selected word.
 */
export function renderJunctionDetails(container, paa, solIndex) {
  if (!paa?.thalaikal?.length) return;

  const relevant = paa.thalaikal.filter(
    t => t.from_sol_index === solIndex || t.to_sol_index === solIndex
  );

  if (relevant.length === 0) return;

  const section = document.createElement('div');
  section.className = 'breakdown-section';

  const title = document.createElement('div');
  title.className = 'breakdown-section-title';
  title.textContent = 'Junctions (தளை)';
  section.appendChild(title);

  relevant.forEach(thalai => {
    const isFrom = thalai.from_sol_index === solIndex;
    const otherIdx = isFrom ? thalai.to_sol_index : thalai.from_sol_index;
    const otherWord = paa.sorkal[otherIdx];

    const item = document.createElement('div');
    item.style.cssText = 'padding: 8px; border: 1px solid var(--color-border-light); border-radius: 4px; margin-bottom: 6px; font-size: 13px;';

    const direction = isFrom
      ? `→ ${otherWord?.raw_text || `Word ${otherIdx + 1}`}`
      : `${otherWord?.raw_text || `Word ${otherIdx + 1}`} →`;

    const eerruLabel = `${titleCase(thalai.eerru_asai)} (${ASAI_NAMES[thalai.eerru_asai] || ''})`;
    const muthalLabel = `${titleCase(thalai.muthal_asai)} (${ASAI_NAMES[thalai.muthal_asai] || ''})`;

    const flags = [];
    if (thalai.is_cross_adi) flags.push('cross-line');
    if (thalai.is_intra_compound) flags.push('intra-compound');
    if (thalai.is_to_eetru) flags.push('to-final-word');

    item.innerHTML = `
      <div style="font-weight: 500; margin-bottom: 4px;">${direction}</div>
      <div style="color: var(--color-text-secondary);">
        ${eerruLabel} → ${muthalLabel}
        ${flags.length ? `<span style="color: var(--color-text-muted);"> · ${flags.join(', ')}</span>` : ''}
      </div>
    `;

    section.appendChild(item);
  });

  container.appendChild(section);
}
