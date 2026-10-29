import {
  lengthMinutesRejected,
  ratingRejected,
  votecountRejected,
} from './form-metadata.js';
import { installPathInvalid } from './installations.js';

/** 校验顺序 = 阅读顺序 = tab 顺序，报错时按此顺序取第一个 */
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

/**
 * 一次算出表单里所有阻塞提交的问题。
 * 返回 { 字段名: i18n 词条 }，空对象表示可以提交。
 * 安装项的报错由列表逐行渲染，这里的键只用于定位到哪个 tab。
 * 只回答「能不能存」，不做归一化——归一化在 form-metadata.js 的键入/写库路径上。
 */
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

/** 出错字段所在的 tab；多个 tab 出错时取最靠前的那个 */
export function errorTab(errors) {
  const field = firstErrorField(errors);
  return field ? FIELD_TAB[field] : null;
}

/** 出错字段该聚焦的输入框 id；安装项没有固定 id，返回 null 表示只切 tab */
export function errorFieldId(errors) {
  const field = firstErrorField(errors);
  return field ? (FIELD_INPUT_ID[field] ?? null) : null;
}
