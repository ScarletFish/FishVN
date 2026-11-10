import '../css/styles.css';
import '../css/dev/visual.css';
import '../lib/side-tip.js';
import { mount } from 'svelte';

import Board from './Board.svelte';

mount(Board, { target: document.getElementById('visual') });
