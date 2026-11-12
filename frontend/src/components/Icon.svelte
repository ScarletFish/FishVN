<script>
  import HouseIcon from 'phosphor-svelte/lib/HouseIcon';
  import BookIcon from 'phosphor-svelte/lib/BookIcon';
  import FunnelIcon from 'phosphor-svelte/lib/FunnelIcon';
  import DownloadIcon from 'phosphor-svelte/lib/DownloadIcon';
  import ChartBarIcon from 'phosphor-svelte/lib/ChartBarIcon';
  import SunIcon from 'phosphor-svelte/lib/SunIcon';
  import MoonIcon from 'phosphor-svelte/lib/MoonIcon';
  import GearIcon from 'phosphor-svelte/lib/GearIcon';
  import CheckCircleIcon from 'phosphor-svelte/lib/CheckCircleIcon';
  import XCircleIcon from 'phosphor-svelte/lib/XCircleIcon';
  import WarningIcon from 'phosphor-svelte/lib/WarningIcon';
  import InfoIcon from 'phosphor-svelte/lib/InfoIcon';
  import BellSlashIcon from 'phosphor-svelte/lib/BellSlashIcon';
  import CaretLeftIcon from 'phosphor-svelte/lib/CaretLeftIcon';
  import CaretRightIcon from 'phosphor-svelte/lib/CaretRightIcon';
  import CaretUpIcon from 'phosphor-svelte/lib/CaretUpIcon';
  import CaretDownIcon from 'phosphor-svelte/lib/CaretDownIcon';
  import FunnelSimpleIcon from 'phosphor-svelte/lib/FunnelSimpleIcon';
  import ArrowUpRightIcon from 'phosphor-svelte/lib/ArrowUpRightIcon';
  import XIcon from 'phosphor-svelte/lib/XIcon';
  import MagnifyingGlassIcon from 'phosphor-svelte/lib/MagnifyingGlassIcon';
  import PlusIcon from 'phosphor-svelte/lib/PlusIcon';
  import CheckIcon from 'phosphor-svelte/lib/CheckIcon';
  import CalendarBlankIcon from 'phosphor-svelte/lib/CalendarBlankIcon';
  import FolderIcon from 'phosphor-svelte/lib/FolderIcon';
  import SquaresFourIcon from 'phosphor-svelte/lib/SquaresFourIcon';
  import PencilSimpleIcon from 'phosphor-svelte/lib/PencilSimpleIcon';
  import PlayIcon from 'phosphor-svelte/lib/PlayIcon';
  import DotsThreeVerticalIcon from 'phosphor-svelte/lib/DotsThreeVerticalIcon';
  import DotsSixVerticalIcon from 'phosphor-svelte/lib/DotsSixVerticalIcon';

  const ICONS = {
    house: HouseIcon,
    book: BookIcon,
    funnel: FunnelIcon,
    download: DownloadIcon,
    'chart-bar': ChartBarIcon,
    sun: SunIcon,
    moon: MoonIcon,
    gear: GearIcon,
    'check-circle': CheckCircleIcon,
    'x-circle': XCircleIcon,
    warning: WarningIcon,
    info: InfoIcon,
    'bell-slash': BellSlashIcon,
    'caret-left': CaretLeftIcon,
    'caret-right': CaretRightIcon,
    'caret-up': CaretUpIcon,
    'caret-down': CaretDownIcon,
    sort: FunnelSimpleIcon,
    'arrow-up-right': ArrowUpRightIcon,
    x: XIcon,
    'magnifying-glass': MagnifyingGlassIcon,
    plus: PlusIcon,
    check: CheckIcon,
    calendar: CalendarBlankIcon,
    folder: FolderIcon,
    'squares-four': SquaresFourIcon,
    pencil: PencilSimpleIcon,
    play: PlayIcon,
    'dots-three': DotsThreeVerticalIcon,
    'dots-six': DotsSixVerticalIcon,
  };

  const SIZE_CLASS = {
    sm: `icon icon--sm`,
    md: `icon icon--md`,
    lg: `icon icon--lg`,
    xl: `icon icon--xl`,
  };

  let { name, size = 'md', class: extra = '' } = $props();

  let C = $derived(ICONS[name]);
</script>

{#if C}
  <C class={`${SIZE_CLASS[size] ?? SIZE_CLASS.md} ${extra}`.trim()} aria-hidden="true" />
{/if}
