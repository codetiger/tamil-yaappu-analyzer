/**
 * word-card.js — Detailed word breakdown panel.
 */
import {
  mapSyllablesToAsai,
  mapGraphemesToSyllables,
  titleCase,
  SEER_NAMES,
  ASAI_NAMES,
  SEER_CATEGORY_NAMES,
  GRAPHEME_TYPE_NAMES,
  VOWEL_LENGTH_NAMES,
} from '../data-mapper.js';
import { renderJunctionDetails } from './junction-view.js';

/**
 * Render the detail panel for a selected word.
 */
export function renderWordDetail(container, word, paa) {
  container.innerHTML = '';
  container.classList.add('visible');

  // Drag handle (visual cue for mobile bottom sheet)
  const dragHandle = document.createElement('div');
  dragHandle.className = 'detail-drag-handle';
  container.appendChild(dragHandle);

  const panel = document.createElement('div');
  panel.className = 'detail-panel';

  // Header with close button
  const header = document.createElement('div');
  header.className = 'detail-header';

  const headerContent = document.createElement('div');
  headerContent.className = 'detail-header-content';

  const wordText = document.createElement('div');
  wordText.className = 'detail-word-text';
  wordText.textContent = word.compound_source_index != null ? word.normalized_text : word.raw_text;
  headerContent.appendChild(wordText);

  const seerInfo = document.createElement('div');
  seerInfo.className = 'detail-seer-info';
  const seerTamil = SEER_NAMES[word.seer_vagai] || '';
  const catTamil = SEER_CATEGORY_NAMES[word.seer_category] || '';
  seerInfo.textContent = `${titleCase(word.seer_vagai)} (${seerTamil}) — ${titleCase(word.seer_category)} (${catTamil})`;
  headerContent.appendChild(seerInfo);

  header.appendChild(headerContent);

  const closeBtn = document.createElement('button');
  closeBtn.className = 'detail-close-btn';
  closeBtn.setAttribute('aria-label', 'Close detail panel');
  closeBtn.innerHTML = '&#215;';
  closeBtn.addEventListener('click', () => {
    container.dispatchEvent(new CustomEvent('detail-close'));
  });
  header.appendChild(closeBtn);

  panel.appendChild(header);

  // Body
  const body = document.createElement('div');
  body.className = 'detail-body';

  renderAsaiBreakdown(body, word);

  if (word.kadai_ezhuthu || word.compound_source_text) {
    renderMetaLines(body, word);
  }

  if (paa && word._solIndex !== undefined) {
    renderJunctionDetails(body, paa, word._solIndex);
  }

  panel.appendChild(body);
  container.appendChild(panel);
}

function renderAsaiBreakdown(body, word) {
  const section = document.createElement('div');
  section.className = 'breakdown-section';

  const title = document.createElement('div');
  title.className = 'breakdown-section-title';
  title.textContent = 'Asai Breakdown (அசை விரிவாக்கம்)';
  section.appendChild(title);

  const syllablesWithGraphemes = mapGraphemesToSyllables(word.ezhuthukkal, word.syllables);
  const asaiWithSyllables = mapSyllablesToAsai(syllablesWithGraphemes, word.asaikal);

  const row = document.createElement('div');
  row.className = 'breakdown-asai-row';

  asaiWithSyllables.forEach((asai) => {
    const box = document.createElement('div');
    const typeLower = asai.vagai || '';
    box.className = `breakdown-asai-box type-${typeLower}`;

    const hdr = document.createElement('div');
    hdr.className = 'breakdown-asai-header';
    const asaiTamil = ASAI_NAMES[asai.vagai] || '';
    hdr.innerHTML = `<span style="color: var(--color-${typeLower})">${titleCase(asai.vagai)}</span> <span style="color: var(--color-text-muted)">(${asaiTamil})</span>`;
    box.appendChild(hdr);

    (asai.syllables || []).forEach(syl => {
      const sylEl = document.createElement('div');
      sylEl.className = 'breakdown-syllable';

      const sylText = document.createElement('div');
      sylText.className = 'breakdown-syllable-text';
      sylText.textContent = syl.text;
      sylEl.appendChild(sylText);

      const sylInfo = document.createElement('div');
      sylInfo.className = 'breakdown-syllable-info';
      const lengthTamil = VOWEL_LENGTH_NAMES[syl.alavu] || '';
      const closedStr = syl.is_closed ? 'closed' : 'open';
      const singleGraphemeLabel = (syl.graphemes?.length === 1)
        ? `${titleCase(syl.graphemes[0].vagai)} (${GRAPHEME_TYPE_NAMES[syl.graphemes[0].vagai] || ''}) · `
        : '';
      sylInfo.textContent = `${singleGraphemeLabel}${titleCase(syl.alavu)} (${lengthTamil}) · ${closedStr} · ${syl.matrai}m`;
      sylEl.appendChild(sylInfo);

      if (syl.graphemes?.length > 1) {
        const gRow = document.createElement('div');
        gRow.className = 'breakdown-graphemes';

        syl.graphemes.forEach(g => {
          const gEl = document.createElement('div');
          const gType = g.vagai || '';
          gEl.className = 'breakdown-grapheme';
          gEl.style.borderLeft = `2px solid var(--color-${gType})`;

          const gText = document.createElement('span');
          gText.textContent = g.text;
          gEl.appendChild(gText);

          const gLabel = document.createElement('div');
          gLabel.className = 'breakdown-grapheme-label';
          const gTamil = GRAPHEME_TYPE_NAMES[g.vagai] || '';
          gLabel.textContent = `${titleCase(g.vagai)} (${gTamil})`;
          gEl.appendChild(gLabel);

          gRow.appendChild(gEl);
        });

        sylEl.appendChild(gRow);
      }

      box.appendChild(sylEl);
    });

    row.appendChild(box);
  });

  section.appendChild(row);
  body.appendChild(section);
}

function renderMetaLines(body, word) {
  const container = document.createElement('div');
  container.className = 'detail-meta-lines';

  if (word.kadai_ezhuthu) {
    const gTamil = GRAPHEME_TYPE_NAMES[word.kadai_ezhuthu_vagai] || '';
    const lenTamil = word.kadai_ezhuthu_alavu ? ` · ${titleCase(word.kadai_ezhuthu_alavu)} (${VOWEL_LENGTH_NAMES[word.kadai_ezhuthu_alavu] || ''})` : '';
    const line = document.createElement('div');
    line.className = 'detail-meta-line';
    line.textContent = `Last: ${word.kadai_ezhuthu} · ${titleCase(word.kadai_ezhuthu_vagai)} (${gTamil})${lenTamil}`;
    container.appendChild(line);
  }

  if (word.compound_source_text) {
    const line = document.createElement('div');
    line.className = 'detail-meta-line';
    line.textContent = `From: ${word.compound_source_text} · Part ${(word.compound_part || 0) + 1}`;
    container.appendChild(line);
  }

  body.appendChild(container);
}
