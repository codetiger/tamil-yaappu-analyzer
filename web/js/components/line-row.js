/**
 * line-row.js — Renders one line row: word boxes + junction connectors + right-side line tags.
 */
import { groupWordsByLine } from '../data-mapper.js';
import { createWordBox } from './word-box.js';

const ADI_TYPE_LABELS = {
  kuraladi: 'குறளடி',
  chinthadi: 'சிந்தடி',
  alavadi: 'அளவடி',
  neduladi: 'நெடிலடி',
  kazhineduladi: 'கழிநெடிலடி',
};

const POSITION_LABELS = {
  first: 'முதல்',
  last: 'இறுதி',
  penultimate: 'ஈற்றயல்',
  middle: 'இடை',
};

const THALAI_NAMES = {
  iyarseer_ventalai: 'இயற்சீர் வெண்டளை',
  venseer_ventalai: 'வெண்சீர் வெண்டளை',
  ner_onru_aasiriyathalai: 'நேரொன்று ஆசிரியத்தளை',
  nirai_onru_aasiriyathalai: 'நிரையொன்று ஆசிரியத்தளை',
  kalithalai: 'கலித்தளை',
  oonru_vanjithalai: 'ஊன்று வஞ்சித்தளை',
  oonraatha_vanjithalai: 'ஊன்றாத வஞ்சித்தளை',
};

/**
 * Render all line rows into the poem body.
 * @param {HTMLElement} container - #poem-body element
 * @param {object} paa - PaaData
 * @param {object} state - { selectedLine }
 * @param {function} onLineClick - callback(lineIdx)
 */
export function renderLineRows(container, paa, state, onLineClick) {
  container.innerHTML = '';
  container.classList.add('visible');

  const lines = groupWordsByLine(paa);
  const adikal = paa.adikal || [];

  // Build lookup: globalSolIndex → enriched word data from adikal.sorkal
  // (words from groupWordsByLine get _solIndex as a global counter matching this)
  const enrichedWordMap = new Map();
  let globalIdx = 0;
  adikal.forEach(adi => {
    const sorkal = adi.sorkal || [];
    sorkal.forEach((sol, j) => {
      enrichedWordMap.set(globalIdx + j, sol);
    });
    globalIdx += sorkal.length;
  });

  lines.forEach((words, lineIdx) => {
    const adi = adikal[lineIdx] || {};
    const row = document.createElement('div');
    row.className = 'line-row';
    row.setAttribute('data-line-idx', lineIdx);

    // Dimming when a line is selected
    if (state.selectedLine !== null && state.selectedLine !== lineIdx) {
      row.classList.add('dimmed');
    }
    if (state.selectedLine === lineIdx) {
      row.classList.add('selected');
    }

    // Line number
    const numEl = document.createElement('div');
    numEl.className = 'line-number';
    numEl.textContent = lineIdx + 1;
    numEl.addEventListener('click', () => onLineClick(lineIdx));
    row.appendChild(numEl);

    // Word boxes area
    const wordsDiv = document.createElement('div');
    wordsDiv.className = 'line-words';

    // Mark last word of last line as eetru
    const isLastLine = lineIdx === lines.length - 1;

    let prevSolIdx = null;

    // For the first word of non-first lines, find the previous line's last word solIndex
    if (lineIdx > 0 && lines[lineIdx - 1].length > 0) {
      const prevLine = lines[lineIdx - 1];
      const lastEntry = prevLine[prevLine.length - 1];
      if (lastEntry._isCompound) {
        const lastPart = lastEntry.parts[lastEntry.parts.length - 1];
        prevSolIdx = lastPart._solIndex;
      } else {
        prevSolIdx = lastEntry._solIndex;
      }
    }

    words.forEach((entry, wordIdx) => {
      const firstSolIdx = entry._isCompound ? entry.parts[0]._solIndex : entry._solIndex;
      // entry already has enriched word data (spread from adi.sorkal[])
      const word = entry._isCompound ? entry.parts[0] : entry;

      // Insert junction connector before this word if it has thalai_from_prev
      if (word && prevSolIdx !== null) {
        const connector = createJunctionConnector(enrichedWordMap, word, firstSolIdx, prevSolIdx);
        if (connector) {
          wordsDiv.appendChild(connector);
        }
      }

      if (entry._isCompound) {
        const group = createCompoundGroup(entry, paa, isLastLine && wordIdx === words.length - 1);
        wordsDiv.appendChild(group);
        // Update prevSolIdx to last part of compound
        const lastPart = entry.parts[entry.parts.length - 1];
        prevSolIdx = lastPart._solIndex;
      } else {
        if (isLastLine && wordIdx === words.length - 1) {
          entry._isEetru = true;
        }
        const box = createWordBox(entry, paa);
        wordsDiv.appendChild(box);
        prevSolIdx = entry._solIndex;
      }
    });

    row.appendChild(wordsDiv);

    // Line-level tags (right side)
    const tagsDiv = document.createElement('div');
    tagsDiv.className = 'line-tags';

    // Word count
    const wordCount = adi.word_count || words.length;
    const wcTag = document.createElement('span');
    wcTag.className = 'line-tag';
    wcTag.setAttribute('data-tag', 'word_count');
    wcTag.textContent = `${wordCount} சொல்`;
    tagsDiv.appendChild(wcTag);

    // Adi type
    const adiType = adi.adi_type?.value || adi.adi_type;
    if (adiType) {
      const atTag = document.createElement('span');
      atTag.className = 'line-tag';
      atTag.setAttribute('data-tag', 'adi_type');
      atTag.textContent = ADI_TYPE_LABELS[adiType] || adiType;
      tagsDiv.appendChild(atTag);
    }

    // Line position
    const pos = adi.line_position;
    if (pos) {
      const posTag = document.createElement('span');
      posTag.className = 'line-tag';
      posTag.setAttribute('data-tag', 'line_position');
      posTag.textContent = POSITION_LABELS[pos] || pos;
      tagsDiv.appendChild(posTag);
    }

    row.appendChild(tagsDiv);
    container.appendChild(row);
  });
}

/**
 * Create a junction connector element between two words.
 * Uses the word's thalai_from_prev and is_ventalai fields (set by a2_thalai workflow).
 * @param {Map} wordMap - globalSolIndex → enriched word data
 * @param {object} word - current word data (the "to" word)
 * @param {number} toSolIdx - solIndex of current word
 * @param {number} fromSolIdx - solIndex of previous word
 */
function createJunctionConnector(wordMap, word, toSolIdx, fromSolIdx) {
  const rawThalai = word.thalai_from_prev;
  const thalaiType = typeof rawThalai === 'object' ? rawThalai?.value : rawThalai;
  const rawValid = word.is_ventalai;
  const isValid = typeof rawValid === 'object' ? rawValid?.value : rawValid;

  // No junction data on this word
  if (thalaiType === null && isValid === null) return null;

  const connector = document.createElement('div');
  connector.className = 'junction-connector';

  const thalaiName = thalaiType ? (THALAI_NAMES[thalaiType] || thalaiType) : null;

  if (isValid === true) {
    connector.classList.add('junction-valid');
    connector.setAttribute('title', 'தளை சரி — முந்தைய சீரின் ஈற்று அசையும் இச்சீரின் முதல் அசையும் பொருந்துகின்றன');
  } else if (isValid === false) {
    connector.classList.add('junction-warning');
    connector.setAttribute('title', 'தளை தவறு — முந்தைய சீரின் ஈற்று அசையும் இச்சீரின் முதல் அசையும் பொருந்தவில்லை');
  } else {
    connector.classList.add('junction-valid');
    connector.setAttribute('title', 'தளை');
  }

  connector.textContent = thalaiName || '—';

  // Junction hover: highlight eerru asai (last of prev word) + muthal asai (first of current word)
  const prevWord = wordMap.get(fromSolIdx);
  const eerruAsaiIdx = prevWord?.asaikal ? prevWord.asaikal.length - 1 : -1;

  if (eerruAsaiIdx >= 0) {
    const showJunctionHighlight = () => {
      const prevBox = document.querySelector(`[data-sol-idx="${fromSolIdx}"]`);
      if (prevBox) {
        prevBox.querySelectorAll(`[data-asai-idx="${eerruAsaiIdx}"]`).forEach(el =>
          el.classList.add('junction-hover-eerru')
        );
      }
      const curBox = document.querySelector(`[data-sol-idx="${toSolIdx}"]`);
      if (curBox) {
        curBox.querySelectorAll('[data-asai-idx="0"]').forEach(el =>
          el.classList.add('junction-hover-muthal')
        );
      }
    };
    const clearJunctionHighlight = () => {
      document.querySelectorAll('.junction-hover-eerru, .junction-hover-muthal').forEach(el =>
        el.classList.remove('junction-hover-eerru', 'junction-hover-muthal')
      );
    };

    connector.addEventListener('mouseenter', showJunctionHighlight);
    connector.addEventListener('mouseleave', clearJunctionHighlight);

    // Touch: toggle highlight on tap
    let junctionActive = false;
    connector.addEventListener('touchend', (e) => {
      e.preventDefault();
      junctionActive = !junctionActive;
      if (junctionActive) { showJunctionHighlight(); } else { clearJunctionHighlight(); }
    });
  }

  return connector;
}

function createCompoundGroup(group, paa, isLastEntry) {
  const wrapper = document.createElement('div');
  wrapper.className = 'compound-group';

  const header = document.createElement('div');
  header.className = 'compound-header';
  header.textContent = group._sourceText;
  wrapper.appendChild(header);

  const partsRow = document.createElement('div');
  partsRow.className = 'compound-parts';

  group.parts.forEach((part, i) => {
    if (isLastEntry && i === group.parts.length - 1) {
      part._isEetru = true;
    }
    const partDiv = document.createElement('div');
    partDiv.className = 'compound-part';
    const box = createWordBox(part, paa);
    partDiv.appendChild(box);
    partsRow.appendChild(partDiv);
  });

  wrapper.appendChild(partsRow);
  return wrapper;
}
