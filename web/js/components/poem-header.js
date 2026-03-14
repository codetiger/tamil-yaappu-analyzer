/**
 * poem-header.js — Renders poem-level classification badge + tag pills.
 */

const PRIMARY_PA_LABELS = {
  venba: 'வெண்பா',
  asiriyappa: 'ஆசிரியப்பா',
  kalippa: 'கலிப்பா',
  vanjippa: 'வஞ்சிப்பா',
  unknown: 'வகைப்படுத்த இயலவில்லை',
};

const GRANULARITY_LABELS = {
  kural_venba: 'குறள் வெண்பா',
  sindhiyal_venba: 'சிந்தியல் வெண்பா',
  nerisai_venba: 'நேரிசை வெண்பா',
  innisai_venba: 'இன்னிசை வெண்பா',
  pahrodai_venba: 'பஃறொடை வெண்பா',
  kali_venba: 'கலி வெண்பா',
  nerisai_asiriyappa: 'நேரிசை ஆசிரியப்பா',
  nilaimandila_asiriyappa: 'நிலைமண்டில ஆசிரியப்பா',
  kalippa: 'கலிப்பா',
  vanjippa: 'வஞ்சிப்பா',
};

const OSAI_LABELS = {
  seppal: 'செப்பல்',
  ahaval: 'அகவல்',
  thullal: 'துள்ளல்',
  thoongal: 'தூங்கல்',
};

const TAG_LABELS = {
  etukai: 'எதுகை',
  monai: 'மோனை',
  iyaipu: 'இயைபு',
  kutrilugaram: 'குற்றியலுகரம்',
  valid_tamil: 'சரியான தமிழ்',
  no_empty_words: 'வெற்றுச்சொல் இல்லை',
  syllabification_ok: 'பிரிப்பு',
  has_overflow: 'மிகை',
  thalai_all_valid: 'தளை',
  eetru_type: 'ஈற்று',
  has_kani_seer: 'கனிச்சீர்',
  link_harmony: 'இணைப்பு இசைவு',
  vikarpam_type: 'விகற்பம்',
};

const TAG_DESCRIPTIONS = {
  etukai: 'ஒவ்வொரு அடியின் முதல் சொல்லின் 2-ஆம் எழுத்து ஒன்றாக இருக்க வேண்டும்',
  monai: 'ஒவ்வொரு அடியின் முதல் சொல்லின் 1-ஆம் எழுத்து ஒன்றாக இருக்க வேண்டும்',
  iyaipu: 'ஒவ்வொரு அடியின் கடைசி சொல்லின் ஒலி ஒன்றாக இருக்க வேண்டும்',
  thalai_all_valid: 'அடுத்தடுத்த சீர்களின் இணைப்பு (தளை) சரியாக உள்ளதா',
  eetru_type: 'பாவின் இறுதிச் சொல்லின் வகை',
  has_overflow: '3 அசைக்கு மேல் கொண்ட சீர் (மிகை) உள்ளதா',
  kutrilugaram: 'இறுதிச் சொல்லில் குற்றியலுகரம் உள்ளதா',
};

const NEGATIVE_TAGS = new Set(['has_overflow', 'has_kani_seer']);

const POEM_TAG_KEYS = [
  'etukai', 'monai', 'iyaipu',
  'thalai_all_valid', 'eetru_type',
  'has_overflow', 'kutrilugaram',
];

/**
 * Render the poem header section.
 * @param {HTMLElement} container - #poem-header element
 * @param {object} analysis - analysis object with classification and tags
 * @param {function} onTagHover - callback(tagKey) on hover
 * @param {function} onTagLeave - callback() on hover leave
 * @param {function} onTagClick - callback(tagKey) on click
 */
export function renderPoemHeader(container, analysis, { onTagHover, onTagLeave, onTagClick }) {
  container.innerHTML = '';
  if (!analysis) return;

  container.classList.add('visible');

  // Top row: classification badge + info
  const topRow = document.createElement('div');
  topRow.className = 'poem-header-top';

  if (analysis.classification) {
    const cls = analysis.classification;

    // Support both old schema (paa_family/venba_type) and new schema (primary_pa.value/granularity_type.value)
    const primaryPa = cls.primary_pa?.value || cls.paa_family || 'unknown';
    const granularity = cls.granularity_type?.value || cls.venba_type || null;
    const osai = cls.osai_type?.value || cls.osai_type || null;

    // Primary badge: granularity type (specific) or primary pa (broad)
    const displayLabel = granularity
      ? (GRANULARITY_LABELS[granularity] || granularity)
      : (PRIMARY_PA_LABELS[primaryPa] || primaryPa);
    const badge = document.createElement('div');
    badge.className = `classification-badge ${primaryPa !== 'unknown' ? 'classified' : 'unclassified'}`;
    badge.textContent = displayLabel;
    topRow.appendChild(badge);

    // Info line: primary pa family + osai + line count
    const infoParts = [];
    if (granularity && primaryPa !== 'unknown') {
      infoParts.push(PRIMARY_PA_LABELS[primaryPa] || primaryPa);
    }
    if (osai && typeof osai === 'string') {
      infoParts.push(OSAI_LABELS[osai] || osai);
    }
    const adiCount = cls.adi_count ?? analysis.tags?.adi_count?.value ?? analysis.tags?.adi_count ?? '?';
    infoParts.push(`${adiCount} அடி`);

    const info = document.createElement('span');
    info.className = 'classification-info';
    info.textContent = infoParts.join(' · ');
    topRow.appendChild(info);
  }

  container.appendChild(topRow);

  // Tag pills row
  if (analysis.tags) {
    const tagsRow = document.createElement('div');
    tagsRow.className = 'poem-tags';

    POEM_TAG_KEYS.forEach(key => {
      const value = analysis.tags[key];
      if (value === undefined) return;

      const pill = createTagPill(key, value, { onTagHover, onTagLeave, onTagClick });
      tagsRow.appendChild(pill);
    });

    container.appendChild(tagsRow);
  }
}

function createTagPill(key, value, { onTagHover, onTagLeave, onTagClick }) {
  const pill = document.createElement('div');
  pill.className = 'tag-pill';
  pill.setAttribute('data-tag-key', key);
  if (TAG_DESCRIPTIONS[key]) {
    pill.setAttribute('title', TAG_DESCRIPTIONS[key]);
  }

  let status;
  if (typeof value === 'boolean') {
    const isNeg = NEGATIVE_TAGS.has(key);
    status = (isNeg ? !value : value) ? 'pass' : 'fail';
  } else {
    status = 'info';
  }
  pill.classList.add(`tag-${status}`);

  const dot = document.createElement('span');
  dot.className = 'tag-dot';
  pill.appendChild(dot);

  const label = document.createElement('span');
  label.className = 'tag-pill-label';
  label.textContent = TAG_LABELS[key] || key;
  pill.appendChild(label);

  if (typeof value === 'string') {
    const val = document.createElement('span');
    val.className = 'tag-pill-value';
    val.textContent = value;
    pill.appendChild(val);
  }

  // On touch devices, skip hover and go straight to click/toggle behavior
  let isTouching = false;
  pill.addEventListener('touchstart', () => { isTouching = true; }, { passive: true });
  pill.addEventListener('touchend', (e) => {
    e.preventDefault(); // prevent mouseenter/click from also firing
    onTagClick(key);
    isTouching = false;
  });

  pill.addEventListener('mouseenter', () => { if (!isTouching) onTagHover(key); });
  pill.addEventListener('mouseleave', () => { if (!isTouching) onTagLeave(); });
  pill.addEventListener('click', () => { if (!isTouching) onTagClick(key); });

  return pill;
}
