# 第三方声明 (Third-Party Notices)

本文件随安装包分发于 `licenses/`，同目录另有 `LICENSE` 与 `OFL.txt`。

## VNDB 数据

Contains information from [VNDB](https://vndb.org), which is made available here under the [Open Database License (ODbL)](https://opendatacommons.org/licenses/odbl/1.0/).

官方 dump 依 ODbL 1.0 + [DbCL 1.0](https://opendatacommons.org/licenses/dbcl/1.0/) 提供，仅用于离线生成，不随包。由它衍生的 `tags_zh.json` / `tag_groups.json` 以 ODbL 提供；机器可读副本与生成脚本见仓库 `src-tauri/resources/vndb/`（`community/build_tag_zh.py` / `build_tag_groups.py`）。

VN 简介、角色简介与图片经 API 运行时按需拉取 + 本地缓存展示，**不打包、不再分发、不得商用**；VNDB 不对这三类授予再分发权（<https://vndb.org/d17#4>、<https://vndb.org/d14#6>）。

## 中文翻译表

`community/vndb-tags-translation.json` 来自 [VNDBTranslatorLib_](https://greasyfork.org/zh-CN/scripts/445990-vndbtranslatorlib)（作者 `rui li 2`，MIT），抽取表由 [PotatoVN](https://github.com/GoldenPotato137/PotatoVN)（Apache-2.0）打包。其 Third Party Notices 原文：

```
Third Party Notices:
------------------

This software includes translation data for VNDB tags from:
- Project: VNDBTranslatorLib (https://greasyfork.org/zh-CN/scripts/445990-vndbtranslatorlib)
- Author: rui li 2
- License: MIT
- File: Assets/Data/vndb-tags-translation.json
```

自补翻译 `community/gap-fill.json` 为 FishVN 独立创作。

### MIT License

```
Copyright (c) rui li 2

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

## 字体与其他

- Noto Sans SC：`Google Inc.`，SIL OFL 1.1，全文见 `licenses/OFL.txt`。
- FishVN 自身代码：MIT，见 `licenses/LICENSE`。
- 其余 Rust crate / npm 包：MIT、Apache-2.0 等宽松许可，许可文本随各自包分发，版本见 `Cargo.lock` / `package-lock.json`。
