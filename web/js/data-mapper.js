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
 * Uses nested paa.adikal[].sorkal[] structure (no flat paa.sorkal).
 * @param {object} paa - PaaData object
 * @returns {Array<Array<object>>} Array of lines, each containing word/group objects
 */
export function groupWordsByLine(paa) {
  if (!paa?.adikal) return [];
  let globalIdx = 0;
  return paa.adikal.map(adi => {
    const sorkal = adi.sorkal || [];
    const result = [];
    let i = 0;
    while (i < sorkal.length) {
      const sol = sorkal[i];
      const word = { ...sol, _solIndex: globalIdx + i };
      if (sol.compound_source_index !== undefined && sol.compound_source_index !== null) {
        const srcIdx = sol.compound_source_index;
        const parts = [word];
        while (i + 1 < sorkal.length) {
          const next = sorkal[i + 1];
          if (next.compound_source_index === srcIdx) {
            parts.push({ ...next, _solIndex: globalIdx + i + 1 });
            i++;
          } else {
            break;
          }
        }
        result.push({
          _isCompound: true,
          _sourceText: sol.compound_source_text || sol.raw || sol.raw_text || '',
          _solIndex: globalIdx + i,
          parts,
        });
      } else {
        result.push(word);
      }
      i++;
    }
    globalIdx += sorkal.length;
    return result;
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

