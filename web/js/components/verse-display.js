/**
 * verse-display.js — Renders the annotated verse with layered overlays.
 */
import {
  groupWordsByLine, titleCase,
  SEER_NAMES,
} from '../data-mapper.js';

/**
 * Render the verse display in the given container.
 */
export function renderVerse(container, paa, onWordClick, selectedIndex) {
  container.innerHTML = '';
  container.classList.add('visible');

  const lines = groupWordsByLine(paa);

  lines.forEach((words, lineIdx) => {
    const lineLabel = document.createElement('div');
    lineLabel.className = 'verse-line-label';
    lineLabel.textContent = `Line ${lineIdx + 1} (அடி)`;
    container.appendChild(lineLabel);

    const lineDiv = document.createElement('div');
    lineDiv.className = 'verse-line';

    words.forEach(entry => {
      if (entry._isCompound) {
        const group = createCompoundGroup(entry, onWordClick, selectedIndex);
        lineDiv.appendChild(group);
      } else {
        const block = createWordBlock(entry, onWordClick, selectedIndex);
        lineDiv.appendChild(block);
      }
    });

    container.appendChild(lineDiv);
  });
}

function createWordBlock(word, onWordClick, selectedIndex) {
  const block = document.createElement('div');
  block.className = 'word-block';
  block.setAttribute('role', 'button');
  block.setAttribute('tabindex', '0');

  const cat = word.seer_category || '';
  if (cat === 'iyarseer') block.classList.add('seer-iyarseer');
  else if (cat === 'venseer') block.classList.add('seer-venseer');
  else if (cat === 'overflow') block.classList.add('seer-overflow');

  if (word._solIndex === selectedIndex) {
    block.classList.add('selected');
  }

  renderSeerLayer(block, word);

  block.addEventListener('click', () => onWordClick(word, word._solIndex));
  block.addEventListener('keydown', (e) => {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      onWordClick(word, word._solIndex);
    }
  });
  return block;
}

function createCompoundGroup(group, onWordClick, selectedIndex) {
  const wrapper = document.createElement('div');
  wrapper.className = 'compound-group';

  // Show original word text as header
  const header = document.createElement('div');
  header.className = 'compound-header';
  header.textContent = group._sourceText;
  wrapper.appendChild(header);

  // Render each sub-part as a smaller word block
  const partsRow = document.createElement('div');
  partsRow.className = 'compound-parts';

  group.parts.forEach(part => {
    const block = createWordBlock(part, onWordClick, selectedIndex);
    block.classList.add('compound-part-block');
    partsRow.appendChild(block);
  });

  wrapper.appendChild(partsRow);
  return wrapper;
}

function displayText(word) {
  return word.compound_source_index != null ? word.normalized_text : word.raw_text;
}

function renderSeerLayer(block, word) {
  const textEl = document.createElement('div');
  textEl.className = 'word-text';
  textEl.textContent = displayText(word);
  block.appendChild(textEl);

  const seerLabel = document.createElement('div');
  seerLabel.className = 'word-seer-label';
  const seerTamil = SEER_NAMES[word.seer_vagai] || '';
  seerLabel.textContent = `${titleCase(word.seer_vagai)} (${seerTamil})`;
  block.appendChild(seerLabel);

}

