import {
  lengthMinutesRejected,
  ratingRejected,
  votecountRejected,
} from './form-metadata.js';
import { installPathInvalid } from './installations.js';

const FIELD_ORDER = ['name', 'rating', 'votecount', 'lengthMinutes', 'installations'];

const FIELD_TAB = {
  name: 'basic',
  rating: 'profile',
  votecount: 'profile',
  lengthMinutes: 'profile',
  installations: 'installations',
};

const FIELD_INPUT_ID = {
  name: 'game-form-name',
  rating: 'game-form-meta-rating',
  votecount: 'game-form-meta-votecount',
  lengthMinutes: 'game-form-meta-length',
};

export function validateGameForm(form) {
  const errors = {};
  if (!(form.name ?? '').trim()) errors.name = 'gameForm.error.nameRequired';
  if (ratingRejected(form.rating ?? '')) errors.rating = 'gameForm.error.ratingRange';
  if (votecountRejected(form.votecount ?? '')) errors.votecount = 'gameForm.error.votecountRange';
  if (lengthMinutesRejected(form.lengthMinutes ?? '')) {
    errors.lengthMinutes = 'gameForm.error.lengthMinutesRange';
  }
  if ((form.installations ?? []).some(installPathInvalid)) {
    errors.installations = 'gameForm.install.pathInvalid';
  }
  return errors;
}

function firstErrorField(errors) {
  return FIELD_ORDER.find((field) => errors[field]) ?? null;
}

export function shownErrors(errors, { submitAttempted = false, touched = {} } = {}) {
  const shown = {};
  for (const [field, key] of Object.entries(errors)) {
    if (submitAttempted || touched[field]) shown[field] = key;
  }
  return shown;
}

export function errorTab(errors) {
  const field = firstErrorField(errors);
  return field ? FIELD_TAB[field] : null;
}

export function errorFieldId(errors) {
  const field = firstErrorField(errors);
  return field ? (FIELD_INPUT_ID[field] ?? null) : null;
}
