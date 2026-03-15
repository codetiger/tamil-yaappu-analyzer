/**
 * app.js — Entry point: WASM loader, orchestration, state management.
 */
import { parseEngineOutput } from './data-mapper.js';
import { renderPoemHeader } from './components/poem-header.js';
import { renderLineRows } from './components/line-row.js';
import { buildEvidenceMap, highlightEvidence, clearEvidence, clearAllHighlights } from './evidence.js';

// Sample verses
const SAMPLE_VERSES = [
  { label: 'Kural Venba — #1', text: 'அகர முதல எழுத்தெல்லாம் ஆதி\nபகவன் முதற்றே உலகு' },
  { label: 'Kural Venba — #47', text: 'எண்ணென்ப ஏனை எழுத்தென்ப இவ்விரண்டும்\nகண்ணென்ப வாழும் உயிர்க்கு' },
  { label: 'Sindhiyal — Innisai', text: 'சுரையாழ அம்மி மிதப்ப வரையனைய\nயானைக்கு நீத்து முயற்கு நிலைஎன்ப\nகானக நாடன் சுனை.' },
  { label: 'Sindhiyal — Nerisai', text: 'முல்லை முறுவலித்துக் காட்டின; மெல்லவே\nசேயிதழ்க் காந்தள் துடுப்பீன்ற; - போயினார்\nதிண்டேர் வரவுரைக்கும் கார்.' },
  { label: 'Nerisai — Oru Vikarpam', text: 'நெல்லுக் கிறைத்தநீர் வாய்க்கால் வழியோடிப்\nபுல்லுக்கு மாங்கே பொசியுமாம் - தொல்லுலகில்\nநல்லா ரொருவர் உளரேல் அவர்பொருட்டு\nஎல்லோர்க்கும் பெய்யும் மழை.' },
  { label: 'Nerisai — Iru Vikarpam', text: 'அஞ்சல் மடவனமே உன்ற னணிநடையும்\nவஞ்சி யனையார் மணிநடையும் - விஞ்சியது\nகாணப் பிடித்ததுகா ணென்றான் களிவண்டு\nமாணப் பிடித்ததார் மன்.' },
  { label: 'Innisai Venba — 1', text: 'அறிதே அறியாமை அந்தம் கடந்த\nசெறிவே செறிவுடை யார்க்கும் - பிறிதோர்\nஉறுதியே யில்லை யுலகில் ஒருவன்\nநெறியே நெறியாய் விடும்.' },
  { label: 'Innisai Venba — 2', text: 'கடைகலக்காற் காயார் கழிகமழ்ஞ் செய்யார்\nகொடையளிக்கண் போச்சாவார் கோலநேர் செய்யார்\nஇடையறுத்துப் போகப் பிறனொருவன் சேரார்\nகடையபாக வாழ்தமென் பார்.' },
  { label: 'Pahrodai — 5 Lines', text: 'தென்னவன் கன்னிச் செழுஞ்சாரல் மாமலைவாய்ப்\nபொன்னிறப் பூவேர் புதுமலராம் - நன்னெறியார்\nஆரம் புனைந்த அம்மணி மேகலை\nபாரம் சுமந்து பயிலுமே - வீரர்க்கும்\nஓசை ஒலியும் உடைத்து' },
  { label: 'Pahrodai — 6 Lines', text: 'வான்மழை பெய்து வழிந்தோடும் வாரிபோல்\nயான்பெற்ற செல்வமும் ஈகையே - வான்பொருளும்\nஇல்லார்க்கு ஈவதே இன்பமென எண்ணுவார்\nநல்லார் ஒருவரே நற்பயனாம் - புல்லார்க்கும்\nஎல்லாம் வழங்கி மகிழ்வதே - மெல்லியல்\nநல்லார் செயல் தரும்' },
  { label: 'Kali Venba — 13 Lines', text: 'பூமேவு செங்கமலப் புத்தேளும் தேறரிய\nபாமேவு தெய்வப் பழமறையும் - தேமேவு\nநாதமும் நாதாந்த முடிவும் நவைதீர்ந்த\nபோதமும் காணாத போதமாய் - ஆதிநடு\nஅந்தம் கடந்தநித்தி யானந்த போதமாய்ப்\nபந்தம் தணந்த பரஞ்சுடராய் - வந்த\nஅடியார் இதயத் தாமரை மேலமர்ந்த\nநெடியான் மருகன் நிமலன் - வடியார்\nவேலோன் மயில்வீரன் வெற்றிப் புயத்தவன்\nகாலோன் வணங்கும் கதிரவன் - மேலோர்\nபுகழும் புகழவன் பொன்னடி போற்றி\nஇகழும் வினைதீர்க்கும் ஈசன் - மகனாய்\nகந்தன் மலரடி போற்றி' },
];

// ===== Theme =====

function initTheme() {
  const stored = localStorage.getItem('theme');
  if (stored === 'light' || stored === 'dark') {
    document.documentElement.setAttribute('data-theme', stored);
  } else {
    const preferred = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
    document.documentElement.setAttribute('data-theme', preferred);
  }
}

function toggleTheme() {
  const current = document.documentElement.getAttribute('data-theme');
  const next = current === 'dark' ? 'light' : 'dark';
  document.documentElement.setAttribute('data-theme', next);
  localStorage.setItem('theme', next);
}

initTheme();

// ===== URL Sharing =====

function encodeShareText(text) {
  return btoa(unescape(encodeURIComponent(text)));
}

function decodeShareText(encoded) {
  try {
    return decodeURIComponent(escape(atob(encoded)));
  } catch {
    return null;
  }
}

function loadFromUrl() {
  const params = new URLSearchParams(window.location.search);
  const encoded = params.get('s');
  if (!encoded) return null;
  const text = decodeShareText(encoded);
  if (!text) return null;
  // Clear the URL param after loading
  window.history.replaceState({}, '', window.location.pathname);
  return text;
}

// State
let engine = null;
let shareTimeout = null;
let currentPaa = null;
let currentAnalysis = null;
let evidenceMap = new Map();
let activeTag = null;

const state = {
  selectedLine: null,
};

// DOM references
const inputSection = document.getElementById('input-section');
const inputTextarea = document.getElementById('input-text');
const btnAnalyze = document.getElementById('btn-analyze');
const btnShare = document.getElementById('btn-share');
const btnThemeToggle = document.getElementById('btn-theme-toggle');
const sampleSelect = document.getElementById('sample-select');
const loadingBar = document.getElementById('loading-bar');
const poemHeader = document.getElementById('poem-header');
const poemBody = document.getElementById('poem-body');

// ===== Initialization =====

async function initWasm() {
  loadingBar.style.display = 'block';
  btnAnalyze.disabled = true;

  try {
    const wasmModule = await import('../pkg/tamil_yaappu_wasm.js');
    await wasmModule.default();
    engine = new wasmModule.TamilProsodyEngine();
    loadingBar.style.display = 'none';
    btnAnalyze.disabled = false;
    if (inputTextarea.value.trim()) {
      analyze();
    }
  } catch (err) {
    loadingBar.innerHTML = `<span style="color: var(--color-error)">Failed to load engine: ${err.message}</span>`;
    console.error('WASM init error:', err);
  }
}

// ===== Analysis =====

async function analyze() {
  const input = inputTextarea.value.trim();
  if (!input || !engine) return;

  btnAnalyze.disabled = true;
  btnAnalyze.textContent = 'Analyzing...';

  try {
    const resultJson = await engine.process(input);
    const { paa, analysis } = parseEngineOutput(resultJson);

    currentPaa = paa;
    currentAnalysis = analysis;
    state.selectedLine = null;
    activeTag = null;

    // Build evidence map
    evidenceMap = buildEvidenceMap(paa, analysis);

    // Enable share button
    btnShare.disabled = false;

    render();
  } catch (err) {
    console.error('Analysis error:', err);
    poemBody.innerHTML = `<div class="loading-bar" style="color: var(--color-error)">Analysis failed: ${err.message || err}</div>`;
    poemBody.classList.add('visible');
  } finally {
    btnAnalyze.disabled = false;
    btnAnalyze.textContent = 'Analyze';
  }
}

// ===== Rendering =====

function render() {
  // Poem header (Level 0)
  renderPoemHeader(poemHeader, currentAnalysis, {
    onTagHover: handleTagHover,
    onTagLeave: handleTagLeave,
    onTagClick: handleTagClick,
  });

  // Line rows with word boxes (Level 1 + 2)
  renderLineRows(poemBody, currentPaa, state, handleLineClick);

  // Re-apply active tag highlight
  if (activeTag) {
    highlightEvidence(evidenceMap, activeTag, true);
  }
}

// ===== Line Selection =====

function handleLineClick(lineIdx) {
  // Toggle
  state.selectedLine = state.selectedLine === lineIdx ? null : lineIdx;
  render();
}

// ===== Tag Interactions =====

function handleTagHover(tagKey) {
  // Show transient highlight on hover, but don't clear a persistent (clicked) tag
  if (activeTag) return;
  clearAllHighlights();
  highlightEvidence(evidenceMap, tagKey, false);
}

function handleTagLeave() {
  // Only clear transient hover highlights, not persistent (clicked) ones
  if (activeTag) return;
  clearEvidence();
}

function handleTagClick(tagKey) {
  // Toggle
  clearAllHighlights();
  if (activeTag === tagKey) {
    activeTag = null;
  } else {
    activeTag = tagKey;
    highlightEvidence(evidenceMap, tagKey, true);
    const pill = document.querySelector(`[data-tag-key="${tagKey}"]`);
    if (pill) pill.classList.add('active');
  }
}

// ===== Keyboard Navigation =====

document.addEventListener('keydown', (e) => {
  if (e.key === 'Escape') {
    if (state.selectedLine !== null) {
      state.selectedLine = null;
      render();
    } else if (activeTag) {
      activeTag = null;
      clearAllHighlights();
    }
  }
});

// When junction/asai touch handlers clear all highlights, reset activeTag state
document.addEventListener('highlights-cleared', () => {
  activeTag = null;
});

// ===== Share =====

function showCopiedFeedback() {
  const span = btnShare.querySelector('span');
  span.textContent = 'Copied!';
  clearTimeout(shareTimeout);
  shareTimeout = setTimeout(() => { span.textContent = 'Share'; }, 2000);
}

async function handleShare() {
  const text = inputTextarea.value.trim();
  if (!text) return;
  const encoded = encodeShareText(text);
  const url = new URL(window.location.href);
  url.search = '';
  url.searchParams.set('s', encoded);
  try {
    await navigator.clipboard.writeText(url.toString());
  } catch {
    // Fallback: select a temporary input
    const tmp = document.createElement('input');
    tmp.value = url.toString();
    document.body.appendChild(tmp);
    tmp.select();
    document.execCommand('copy');
    document.body.removeChild(tmp);
  }
  showCopiedFeedback();
}

// ===== Event Wiring =====

btnAnalyze.addEventListener('click', analyze);
btnShare.addEventListener('click', handleShare);
btnThemeToggle.addEventListener('click', toggleTheme);

inputTextarea.addEventListener('keydown', (e) => {
  if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) {
    e.preventDefault();
    analyze();
  }
});

sampleSelect.addEventListener('change', () => {
  const idx = parseInt(sampleSelect.value, 10);
  if (idx >= 0 && idx < SAMPLE_VERSES.length) {
    inputTextarea.value = SAMPLE_VERSES[idx].text;
    if (engine) analyze();
  }
});

// Populate sample select
SAMPLE_VERSES.forEach((k, i) => {
  const opt = document.createElement('option');
  opt.value = i;
  opt.textContent = k.label;
  sampleSelect.appendChild(opt);
});

// Load from shared URL or pre-fill with Kural #1
const sharedText = loadFromUrl();
if (sharedText) {
  inputTextarea.value = sharedText;
} else {
  inputTextarea.value = SAMPLE_VERSES[0].text;
}

// Set titlebar height CSS variable for sticky poem-header
const titlebar = document.querySelector('.titlebar');
if (titlebar) {
  const ro = new ResizeObserver(([entry]) => {
    document.documentElement.style.setProperty('--titlebar-height', entry.target.offsetHeight + 'px');
  });
  ro.observe(titlebar);
}

// Start
initWasm();
