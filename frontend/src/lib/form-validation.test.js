import { describe, expect, it } from 'vitest';
import { errorFieldId, errorTab, validateGameForm } from './form-validation.js';
import { newInstallation } from './installations.js';

const base = {
  name: '千恋万花',
  rating: '88.5',
  votecount: '19334',
  lengthMinutes: '1820',
  installations: [],
};

describe('validateGameForm', () => {
  it('合法表单返回空对象', () => {
    expect(validateGameForm(base)).toEqual({});
  });

  it('名字全空白算缺失', () => {
    expect(validateGameForm({ ...base, name: '   ' })).toEqual({
      name: 'gameForm.error.nameRequired',
    });
  });

  it('评分边界：0 与 100 通过，101 与负数拒绝', () => {
    expect(validateGameForm({ ...base, rating: '0' })).toEqual({});
    expect(validateGameForm({ ...base, rating: '100' })).toEqual({});
    expect(validateGameForm({ ...base, rating: '101' })).toEqual({
      rating: 'gameForm.error.ratingRange',
    });
    expect(validateGameForm({ ...base, rating: '-1' })).toEqual({
      rating: 'gameForm.error.ratingRange',
    });
  });

  it('留空的数值字段不报错', () => {
    expect(
      validateGameForm({ ...base, rating: '', votecount: '', lengthMinutes: '' })
    ).toEqual({});
  });

  it('评分人数与时长拒绝负数与非数字', () => {
    expect(validateGameForm({ ...base, votecount: '-3' })).toHaveProperty('votecount');
    expect(validateGameForm({ ...base, lengthMinutes: '很久' })).toHaveProperty('lengthMinutes');
  });

  it('安装项路径为空算错', () => {
    const draft = newInstallation('  ');
    expect(validateGameForm({ ...base, installations: [draft] })).toEqual({
      installations: 'gameForm.install.pathInvalid',
    });
  });

  it('安装项有可解析路径时通过', () => {
    const draft = newInstallation('D:\\Games\\Senren\\senren.exe');
    expect(validateGameForm({ ...base, installations: [draft] })).toEqual({});
  });

  it('多个错误一次报全', () => {
    const errors = validateGameForm({
      ...base,
      name: '',
      rating: '101',
      votecount: '-1',
      installations: [newInstallation('')],
    });
    expect(Object.keys(errors).sort()).toEqual(['installations', 'name', 'rating', 'votecount']);
  });
});

describe('错误定位', () => {
  it('多 tab 出错时跳到最靠前的 tab', () => {
    expect(errorTab({ installations: 'x', rating: 'y' })).toBe('profile');
    expect(errorTab({ name: 'x', rating: 'y' })).toBe('basic');
  });

  it('只有安装项出错时落在安装 tab 且没有可聚焦输入框', () => {
    const errors = { installations: 'x' };
    expect(errorTab(errors)).toBe('installations');
    expect(errorFieldId(errors)).toBeNull();
  });

  it('给出该聚焦的输入框 id', () => {
    expect(errorFieldId({ name: 'x' })).toBe('game-form-name');
    expect(errorFieldId({ rating: 'y', votecount: 'z' })).toBe('game-form-meta-rating');
    expect(errorFieldId({})).toBeNull();
  });
});
