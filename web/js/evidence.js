/**
 * evidence.js — Evidence highlighting engine.
 * Maps tag keys to DOM selectors and manages highlight/clear.
 */

/**
 * Build evidence map: tag key → array of CSS selectors to highlight.
 * @param {object} paa - PaaData
 * @param {object} analysis - analysis object with tags
 * @returns {Map<string, string[]>} tag key → selectors
 */
export function buildEvidenceMap(paa, analysis) {
  const map = new Map();
  if (!paa || !analysis?.tags) return map;

  const tags = analysis.tags;

  // etukai → highlight irandaam_ezhuthu (2nd letter) of first word of each line
  if (tags.etukai !== undefined) {
    const selectors = [];
    paa.adikal?.forEach((_, i) => {
      selectors.push(`[data-line-idx="${i}"] .word-box:first-of-type [data-grapheme="irandaam"]`);
      selectors.push(`[data-line-idx="${i}"] .compound-group:first-child .word-box:first-of-type [data-grapheme="irandaam"]`);
    });
    map.set('etukai', selectors);
  }

  // monai → highlight muthal_ezhuthu (1st letter) of first word of each line
  if (tags.monai !== undefined) {
    const selectors = [];
    paa.adikal?.forEach((_, i) => {
      selectors.push(`[data-line-idx="${i}"] .word-box:first-of-type [data-grapheme="muthal"]`);
      selectors.push(`[data-line-idx="${i}"] .compound-group:first-child .word-box:first-of-type [data-grapheme="muthal"]`);
    });
    map.set('monai', selectors);
  }

  // iyaipu → highlight last word of each line
  if (tags.iyaipu !== undefined) {
    const selectors = [];
    paa.adikal?.forEach((_, i) => {
      selectors.push(`[data-line-idx="${i}"] .word-box:last-of-type`);
    });
    map.set('iyaipu', selectors);
  }

  // thalai_all_valid → highlight all junction tags
  if (tags.thalai_all_valid !== undefined) {
    map.set('thalai_all_valid', ['.junction-connector.junction-valid', '.junction-connector.junction-warning', '.junction-connector.junction-crossline']);
  }

  // sol_per_adi → highlight line tags showing word count
  if (tags.sol_per_adi !== undefined) {
    map.set('sol_per_adi', ['.line-tag[data-tag="word_count"]']);
  }

  // has_overflow → highlight overflow word boxes
  if (tags.has_overflow !== undefined) {
    map.set('has_overflow', ['.word-box.seer-overflow']);
  }

  // eetru_type → highlight the last word box
  if (tags.eetru_type !== undefined) {
    const lastLineIdx = (paa.adikal?.length || 1) - 1;
    map.set('eetru_type', [`[data-line-idx="${lastLineIdx}"] .word-box:last-of-type`]);
  }

  // kutrilugaram → highlight words with kutriyalukaram + their kutri badges
  if (tags.kutrilugaram !== undefined) {
    map.set('kutrilugaram', ['.word-box[data-kutri="true"]', '.word-tag.tag-kutri']);
  }

  return map;
}

/**
 * Highlight evidence elements for a given tag key.
 * @param {Map<string, string[]>} evidenceMap
 * @param {string} tagKey
 * @param {boolean} persistent - if true, use 'evidence-selected' class
 */
export function highlightEvidence(evidenceMap, tagKey, persistent = false) {
  const selectors = evidenceMap.get(tagKey);
  if (!selectors) return;

  const cls = persistent ? 'evidence-selected' : 'evidence-highlight';
  selectors.forEach(sel => {
    document.querySelectorAll(sel).forEach(el => el.classList.add(cls));
  });
}

/**
 * Clear all evidence highlights.
 * @param {boolean} persistentOnly - if true, only clear 'evidence-selected'
 */
export function clearEvidence(persistentOnly = false) {
  document.querySelectorAll('.evidence-highlight').forEach(el => el.classList.remove('evidence-highlight'));
  if (!persistentOnly) {
    document.querySelectorAll('.evidence-selected').forEach(el => el.classList.remove('evidence-selected'));
  }
}
