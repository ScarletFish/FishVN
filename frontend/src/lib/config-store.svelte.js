import { api } from './api.js';
import { CONFIG_CHANGED } from './config-events.js';

let hideR18Tags = $state(false);
let spoilerLevel = $state('hide_all');
let excludedTags = $state([]);
let timeMetric = $state('foreground');

export function getHideR18Tags() {
  return hideR18Tags;
}

export function getSpoilerLevel() {
  return spoilerLevel;
}

export function getExcludedTags() {
  return excludedTags;
}

export function getTimeMetric() {
  return timeMetric;
}

let inflight = null;

export function loadConfig() {
  inflight ??= api
    .getConfig()
    .then((config) => {
      if (typeof config?.hide_r18_tags === 'boolean') hideR18Tags = config.hide_r18_tags;
      if (typeof config?.spoiler_level === 'string') spoilerLevel = config.spoiler_level;
      if (Array.isArray(config?.excluded_tags)) excludedTags = config.excluded_tags;
      timeMetric = config?.time_metric === 'alive' ? 'alive' : 'foreground';
    })
    .catch(() => {})
    .finally(() => {
      inflight = null;
    });
  return inflight;
}

if (typeof window !== 'undefined') {
  window.addEventListener(CONFIG_CHANGED, loadConfig);
}
