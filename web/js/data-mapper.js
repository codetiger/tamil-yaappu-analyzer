/**
 * data-mapper.js — Transform raw WASM output (Message JSON) into UI-friendly structures.
 */

/**
 * Parse the raw Message JSON string from the WASM engine.
 * @param {string} jsonString - Raw JSON from engine.process()
 * @returns {{ paa: object, analysis: object, auditTrail: Array }}
 */
export function parseEngineOutput(jsonString) {
  const message = JSON.parse(jsonString);
  const paa = message?.context?.data?.paa ?? null;
  const analysis = message?.context?.data?.analysis ?? null;
  const auditTrail = message?.audit_trail ?? [];
  return { paa, analysis, auditTrail };
}

/**
 * Group words (sorkal) by line (adi), merging compound sub-parts.
 * Compound parts sharing the same compound_source_index are grouped
 * into a single display entry with a `parts` array.
 * @param {object} paa - PaaData object
 * @returns {Array<Array<object>>} Array of lines, each containing word/group objects
 */
export function groupWordsByLine(paa) {
  if (!paa?.adikal || !paa?.sorkal) return [];
  return paa.adikal.map(adi => {
    const result = [];
    let i = 0;
    const indices = adi.sol_varisaikal;
    while (i < indices.length) {
      const sol = paa.sorkal[indices[i]];
      const word = { ...sol, _solIndex: indices[i] };
      if (sol.compound_source_index !== undefined && sol.compound_source_index !== null) {
        // Start of a compound group — collect all parts with same source
        const srcIdx = sol.compound_source_index;
        const parts = [word];
        while (i + 1 < indices.length) {
          const next = paa.sorkal[indices[i + 1]];
          if (next.compound_source_index === srcIdx) {
            parts.push({ ...next, _solIndex: indices[i + 1] });
            i++;
          } else {
            break;
          }
        }
        result.push({
          _isCompound: true,
          _sourceText: sol.compound_source_text || sol.raw_text,
          _solIndex: indices[i], // use last part index for selection
          parts,
        });
      } else {
        result.push(word);
      }
      i++;
    }
    return result;
  });
}

/**
 * Map syllables to asai groups for a given word.
 * @param {Array} syllables - SyllableData array
 * @param {Array} asaikal - AsaiData array
 * @returns {Array<{ vagai: string, text: string, syllables: Array }>}
 */
export function mapSyllablesToAsai(syllables, asaikal) {
  if (!syllables?.length || !asaikal?.length) return [];
  let sylIdx = 0;
  return asaikal.map(asai => {
    const syls = [];
    let accumulated = '';
    while (sylIdx < syllables.length && accumulated.length < asai.text.length) {
      syls.push(syllables[sylIdx]);
      accumulated += syllables[sylIdx].text;
      sylIdx++;
    }
    return { ...asai, syllables: syls };
  });
}

/**
 * Map graphemes (ezhuthukkal) to syllables for a given word.
 * @param {Array} ezhuthukkal - EzhuthuData array
 * @param {Array} syllables - SyllableData array
 * @returns {Array<{ text: string, alavu: string, is_closed: boolean, matrai: number, graphemes: Array }>}
 */
export function mapGraphemesToSyllables(ezhuthukkal, syllables) {
  if (!ezhuthukkal?.length || !syllables?.length) return [];
  let gIdx = 0;
  return syllables.map(syl => {
    const graphemes = [];
    let accumulated = '';
    while (gIdx < ezhuthukkal.length && accumulated.length < syl.text.length) {
      graphemes.push(ezhuthukkal[gIdx]);
      accumulated += ezhuthukkal[gIdx].text;
      gIdx++;
    }
    return { ...syl, graphemes };
  });
}

/**
 * Convert snake_case to Title Case for display.
 * e.g. "uyirmei" -> "Uyirmei", "thema" -> "Thema"
 */
export function titleCase(s) {
  if (!s) return '';
  return s.charAt(0).toUpperCase() + s.slice(1);
}

// ===== Lookup maps =====
// Keys are snake_case to match Rust serde serialization

/** Tamil display names for seer types */
export const SEER_NAMES = {
  thema: 'தேமா',
  pulima: 'புளிமா',
  koovilam: 'கூவிளம்',
  karuvilam: 'கருவிளம்',
  themangai: 'தேமாங்காய்',
  themangani: 'தேமாங்கனி',
  koovilankai: 'கூவிளங்காய்',
  koovilankani: 'கூவிளங்கனி',
  pulimangai: 'புளிமாங்காய்',
  pulimangani: 'புளிமாங்கனி',
  karuvilangai: 'கருவிளங்காய்',
  karuvilankani: 'கருவிளங்கனி',
  overflow: 'மிகை',
};

/** Tamil display names for asai types */
export const ASAI_NAMES = {
  neer: 'நேர்',
  nirai: 'நிரை',
};

/** Tamil display names for seer categories */
export const SEER_CATEGORY_NAMES = {
  iyarseer: 'இயற்சீர்',
  venseer: 'வெண்சீர்',
  overflow: 'மிகை',
};

/** Tamil display names for grapheme types */
export const GRAPHEME_TYPE_NAMES = {
  uyir: 'உயிர்',
  mei: 'மெய்',
  uyirmei: 'உயிர்மெய்',
  aytham: 'ஆய்தம்',
};

/** Tamil display names for vowel length */
export const VOWEL_LENGTH_NAMES = {
  kuril: 'குறில்',
  nedil: 'நெடில்',
};

/**
 * Categorize errors by severity based on code prefix.
 * @param {Array} errors - Array of { code, message }
 * @returns {{ errors: Array, warnings: Array, infos: Array }}
 */
export function categorizeErrors(errors) {
  const errs = [];
  const warns = [];
  const infos = [];
  (errors || []).forEach(e => {
    const code = e.code || '';
    if (code.startsWith('E_')) errs.push(e);
    else if (code.startsWith('W_')) warns.push(e);
    else infos.push(e);
  });
  return { errors: errs, warnings: warns, infos };
}
