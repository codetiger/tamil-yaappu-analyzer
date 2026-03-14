/**
 * app.js — Entry point: WASM loader, orchestration, state management.
 */
import { parseEngineOutput } from './data-mapper.js';
import { renderPoemHeader } from './components/poem-header.js';
import { renderLineRows } from './components/line-row.js';
import { buildEvidenceMap, highlightEvidence, clearEvidence } from './evidence.js';

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

// State
let engine = null;
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
  if (activeTag) {
    activeTag = null;
    document.querySelectorAll('.tag-pill.active').forEach(el => el.classList.remove('active'));
  }
  clearEvidence();
  highlightEvidence(evidenceMap, tagKey, false);
}

function handleTagLeave() {
  clearEvidence();
}

function handleTagClick(tagKey) {
  // Toggle
  if (activeTag === tagKey) {
    activeTag = null;
    clearEvidence();
    // Remove active class from pills
    document.querySelectorAll('.tag-pill.active').forEach(el => el.classList.remove('active'));
  } else {
    activeTag = tagKey;
    clearEvidence();
    highlightEvidence(evidenceMap, tagKey, true);
    // Update pill active states
    document.querySelectorAll('.tag-pill.active').forEach(el => el.classList.remove('active'));
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
      clearEvidence();
      document.querySelectorAll('.tag-pill.active').forEach(el => el.classList.remove('active'));
    }
  }
});

// ===== Event Wiring =====

btnAnalyze.addEventListener('click', analyze);

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

// Pre-fill with Kural #1
inputTextarea.value = SAMPLE_VERSES[0].text;

// Start
initWasm();
