<!--
SPDX-FileCopyrightText: 2025 Chen Linxuan <me@black-desk.cn>

SPDX-License-Identifier: MIT
-->

# git-backport

[![checks][badge-shields-io-checks]][actions]
[![commit activity][badge-shields-io-commit-activity]][commits]
[![contributors][badge-shields-io-contributors]][contributors]
[![release date][badge-shields-io-release-date]][releases]
![commits since release][badge-shields-io-commits-since-release]
[![codecov][badge-shields-io-codecov]][codecov]

[badge-shields-io-checks]:
  https://img.shields.io/github/check-runs/black-desk/git-backport/master

[actions]: https://github.com/black-desk/git-backport/actions

[badge-shields-io-commit-activity]:
  https://img.shields.io/github/commit-activity/w/black-desk/git-backport/master

[commits]: https://github.com/black-desk/git-backport/commits/master

[badge-shields-io-contributors]:
  https://img.shields.io/github/contributors/black-desk/git-backport

[contributors]: https://github.com/black-desk/git-backport/graphs/contributors

[badge-shields-io-release-date]:
  https://img.shields.io/github/release-date/black-desk/git-backport

[releases]: https://github.com/black-desk/git-backport/releases

[badge-shields-io-commits-since-release]:
  https://img.shields.io/github/commits-since/black-desk/git-backport/latest

[badge-shields-io-codecov]:
  https://codecov.io/github/black-desk/git-backport/graph/badge.svg?token=6TSVGQ4L9X
[codecov]: https://codecov.io/github/black-desk/git-backport

## 使用

一个简单的git命令行扩展，提供了一些backport过程中常用的功能，具体使用见`help`子命令的输出：

```bash
git bp help
```

## 许可证

如无特殊说明，该项目的代码以GNU通用公共许可协议第三版或任何更新的版本开源，文档、配置文件以及开发维护过程中使用的脚本等以MIT许可证开源。

该项目遵守[REUSE规范]。

你可以使用[reuse-tool](https://github.com/fsfe/reuse-tool)生成这个项目的SPDX列表：

```bash
reuse spdx
```

[REUSE规范]: https://reuse.software/spec-3.3/
