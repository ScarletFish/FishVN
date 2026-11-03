import { emptyQueryUi } from './query-spec.js';

export const queryStore = $state({
  ui: emptyQueryUi(),
  results: [],
  resultsSubject: 'game',
  resultsCache: { game: [], character: [] },
  sortBy: 'vndb',
  lastJson: { game: null, character: null },
});
