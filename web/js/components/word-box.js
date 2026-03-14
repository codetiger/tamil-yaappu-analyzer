/**
 * word-box.js — Renders a single word box with embedded tags.
 */
import { titleCase, SEER_NAMES, ASAI_NAMES } from '../data-mapper.js';
import { clearAllHighlights } from '../evidence.js';

// Map seer_group values to CSS class names for border color
const SEER_GROUP_TO_CLASS = {
  ma_seer: 'seer-iyarseer',
  vila_seer: 'seer-iyarseer',
  kaai_seer: 'seer-venseer',
  kani_seer: 'seer-venseer',
};

/**
 * Create a word box element.
 * @param {object} word - word data with _solIndex (from adikal[].sorkal[])
 * @param {object} paa - PaaData (unused now, kept for API compat)
 * @returns {HTMLElement}
 */
export function createWordBox(word, paa) {
  const box = document.createElement('div');
  box.className = 'word-box';
  box.setAttribute('data-sol-idx', word._solIndex);

  // Seer group border color
  const seerGroup = word.seer_group?.value || word.seer_group || '';
  const seerClass = SEER_GROUP_TO_CLASS[seerGroup];
  if (seerClass) {
    box.classList.add(seerClass);
  } else if (word.asai_count > 3) {
    box.classList.add('seer-overflow');
  }

  // Kutriyalukaram data attribute
  const isKutri = word.is_kutriyalukaram === true || word.is_kutriyalukaram?.value === true;
  if (isKutri) {
    box.setAttribute('data-kutri', 'true');
  }

  // Tamil word text with grapheme spans for letter-level highlighting
  const displayText = word.normalized_text || word.raw_text || word.raw || '';
  const textEl = document.createElement('div');
  textEl.className = 'word-box-text';

  const rendered = renderWordTextWithSpans(word, displayText);
  if (rendered.html) {
    textEl.innerHTML = rendered.html;
  } else {
    textEl.textContent = displayText;
  }
  box.appendChild(textEl);

  // Asai pattern strip with hover highlighting
  if (word.asaikal?.length) {
    const strip = document.createElement('div');
    strip.className = 'asai-strip';
    word.asaikal.forEach((asai, asaiIdx) => {
      const chip = document.createElement('span');
      const typeLower = asai.vagai || '';
      chip.className = `asai-chip type-${typeLower}`;
      chip.textContent = ASAI_NAMES[asai.vagai] || titleCase(asai.vagai);

      const showAsaiHighlight = () => {
        const hoverClass = `asai-hover-${typeLower}`;
        box.querySelectorAll(`[data-asai-idx="${asaiIdx}"]`).forEach(el => el.classList.add(hoverClass));
      };
      const clearAsaiHighlight = () => {
        box.querySelectorAll('[data-asai-idx]').forEach(el => {
          el.classList.remove('asai-hover-neer', 'asai-hover-nirai');
        });
      };

      chip.addEventListener('mouseenter', showAsaiHighlight);
      chip.addEventListener('mouseleave', clearAsaiHighlight);

      // Touch: tap to highlight, tap again to clear; clears all other highlights page-wide
      chip.addEventListener('touchend', (e) => {
        e.preventDefault();
        const wasActive = chip.classList.contains('asai-active');
        clearAllHighlights();
        document.dispatchEvent(new CustomEvent('highlights-cleared'));
        if (!wasActive) {
          chip.classList.add('asai-active');
          showAsaiHighlight();
        }
      });

      strip.appendChild(chip);
    });
    box.appendChild(strip);
  }

  // Seer name (vaaippaadu)
  const seerVal = word.vaaippaadu?.value || word.vaaippaadu || word.seer_vagai || null;
  if (seerVal) {
    const seerEl = document.createElement('div');
    seerEl.className = 'word-box-seer';
    const seerTamil = SEER_NAMES[seerVal] || '';
    seerEl.textContent = seerTamil || titleCase(seerVal);
    box.appendChild(seerEl);
  }

  // Word-level tags container
  const tagsDiv = document.createElement('div');
  tagsDiv.className = 'word-box-tags';

  // Kutriyalukaram badge
  if (isKutri) {
    const kutriTag = document.createElement('span');
    kutriTag.className = 'word-tag tag-kutri';
    kutriTag.textContent = 'குற்று';
    kutriTag.setAttribute('title', 'குற்றியலுகரம் — இறுதி எழுத்து குறுகி ஒலிக்கும்');
    tagsDiv.appendChild(kutriTag);
  }

  // Eetru marker (for last word of last line)
  if (word._isEetru) {
    const eetruTag = document.createElement('span');
    eetruTag.className = 'word-tag tag-eetru';
    eetruTag.textContent = 'ஈற்று';
    eetruTag.setAttribute('title', 'ஈற்றுச் சீர் — பாவின் இறுதிச் சொல்');
    tagsDiv.appendChild(eetruTag);
  }

  if (tagsDiv.children.length > 0) {
    box.appendChild(tagsDiv);
  }

  return box;
}

function escapeHtml(str) {
  const div = document.createElement('div');
  div.textContent = str;
  return div.innerHTML;
}

/**
 * Segment Tamil text into grapheme clusters.
 * A Tamil grapheme cluster is: base (consonant/vowel/aytham) + optional vowel sign + optional pulli/virama.
 * This prevents splitting vowel signs from their base consonant.
 */
function segmentTamilGraphemes(text) {
  // Use Intl.Segmenter if available (modern browsers), otherwise regex fallback
  if (typeof Intl !== 'undefined' && Intl.Segmenter) {
    const segmenter = new Intl.Segmenter('ta', { granularity: 'grapheme' });
    return [...segmenter.segment(text)].map(s => s.segment);
  }
  // Fallback: Tamil base char + optional combining marks (vowel signs, virama, etc.)
  const re = /[\u0B80-\u0BFF][\u0BBE-\u0BCD\u0BD7]*/g;
  const result = [];
  let m;
  while ((m = re.exec(text)) !== null) {
    result.push(m[0]);
  }
  return result.length > 0 ? result : [...text];
}

/**
 * Render word text with spans for muthal/irandaam grapheme highlighting
 * and asai-index mapping for asai chip hover.
 * Splits at Tamil grapheme cluster boundaries to avoid breaking ligatures.
 */
function renderWordTextWithSpans(word, displayText) {
  if (!displayText) return { html: null, text: '' };

  const muthal = word.muthal_ezhuthu || null;
  const irandaam = word.irandaam_ezhuthu || null;
  const asaikal = word.asaikal || [];

  // Segment into proper Tamil grapheme clusters
  const graphemes = segmentTamilGraphemes(displayText);
  if (graphemes.length === 0) return { html: null, text: displayText };

  // Build grapheme-index to asai-index mapping using asai text lengths
  const asaiMap = new Array(graphemes.length).fill(-1);
  let gIdx = 0;
  for (let asaiIdx = 0; asaiIdx < asaikal.length; asaiIdx++) {
    const asaiText = asaikal[asaiIdx].text;
    let accumulated = '';
    while (gIdx < graphemes.length && accumulated.length < asaiText.length) {
      asaiMap[gIdx] = asaiIdx;
      accumulated += graphemes[gIdx];
      gIdx++;
    }
  }

  // Check if first grapheme cluster starts with muthal_ezhuthu
  const isMuthal = (i) => i === 0 && muthal && graphemes[0].startsWith(muthal);
  const isIrandaam = (i) => i === 1 && irandaam && graphemes[1].startsWith(irandaam);

  let html = '';
  graphemes.forEach((g, i) => {
    const escaped = escapeHtml(g);
    const asaiAttr = asaiMap[i] >= 0 ? ` data-asai-idx="${asaiMap[i]}"` : '';

    if (isMuthal(i)) {
      html += `<span data-grapheme="muthal"${asaiAttr}>${escaped}</span>`;
    } else if (isIrandaam(i)) {
      html += `<span data-grapheme="irandaam"${asaiAttr}>${escaped}</span>`;
    } else {
      html += `<span${asaiAttr}>${escaped}</span>`;
    }
  });

  return { html, text: null };
}
