/**
 * validation-panel.js — Renders validation diagnostics and ornamentation summary.
 */
import { categorizeErrors } from '../data-mapper.js';

/**
 * Render the validation panel.
 * @param {HTMLElement} container - The validation zone element
 * @param {Array} errors - Array of { code, message } from engine output
 * @param {object|null} ani - AniData object (ornamentation)
 */
export function renderValidation(container, errors, ani) {
  container.innerHTML = '';

  // Ornamentation summary only
  if (ani) {
    container.classList.add('visible');
    renderAniSummary(container, ani);
  }
}

function renderSummary(container, errs, warns, infos) {
  const summary = document.createElement('div');
  summary.className = 'validation-summary';

  if (errs.length === 0 && warns.length === 0) {
    const badge = createBadge('pass', 'Valid');
    summary.appendChild(badge);
  }

  if (errs.length > 0) {
    summary.appendChild(createBadge('error', `${errs.length} Error${errs.length > 1 ? 's' : ''}`));
  }
  if (warns.length > 0) {
    summary.appendChild(createBadge('warning', `${warns.length} Warning${warns.length > 1 ? 's' : ''}`));
  }
  if (infos.length > 0) {
    summary.appendChild(createBadge('info', `${infos.length} Info`));
  }

  container.appendChild(summary);
}

function createBadge(severity, text) {
  const badge = document.createElement('span');
  badge.className = `validation-badge severity-${severity}`;
  badge.textContent = text;
  return badge;
}

function renderDiagnosticList(container, errors) {
  const list = document.createElement('div');
  list.className = 'validation-list';

  errors.forEach(err => {
    const code = err.code || '';
    let severity = 'info';
    if (code.startsWith('E_')) severity = 'error';
    else if (code.startsWith('W_')) severity = 'warning';

    const item = document.createElement('div');
    item.className = `validation-item severity-${severity}`;

    const sevLabel = document.createElement('span');
    sevLabel.className = `validation-severity severity-${severity}`;
    sevLabel.textContent = severity === 'error' ? 'ERR' : severity === 'warning' ? 'WARN' : 'INFO';
    item.appendChild(sevLabel);

    const content = document.createElement('div');
    content.className = 'validation-content';

    const codeEl = document.createElement('div');
    codeEl.className = 'validation-code';
    codeEl.textContent = code;
    content.appendChild(codeEl);

    const msgEl = document.createElement('div');
    msgEl.className = 'validation-message';
    msgEl.textContent = err.message || '';
    content.appendChild(msgEl);

    item.appendChild(content);
    list.appendChild(item);
  });

  container.appendChild(list);
}

function renderAniSummary(container, ani) {
  const summary = document.createElement('div');
  summary.className = 'ani-summary';

  const items = [
    { key: 'etukai_present', label: 'Etukai (எதுகை)', desc: '2nd grapheme match' },
    { key: 'monai_present', label: 'Monai (மோனை)', desc: 'First letter alliteration' },
    { key: 'iyaipu_present', label: 'Iyaipu (இயைபு)', desc: 'End sound rhyme' },
  ];

  items.forEach(({ key, label, desc }) => {
    const card = document.createElement('div');
    const present = ani[key] === true;
    card.className = `ani-card ${present ? 'present' : 'absent'}`;

    const labelEl = document.createElement('div');
    labelEl.className = 'ani-card-label';
    labelEl.textContent = label;
    card.appendChild(labelEl);

    const status = document.createElement('div');
    status.className = 'ani-card-status';
    status.textContent = present ? 'Present' : 'Absent';
    card.appendChild(status);

    summary.appendChild(card);
  });

  container.appendChild(summary);
}
