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

en | [zh_CN](README.zh_CN.md)

> [!WARNING]
>
> This English README is translated from the Chinese version using LLM and may
> contain errors.

## Usage

A simple git command-line extension that provides commonly used features during the backport process. For specific usage, see the output of the `help` subcommand:

```bash
git bp help
```

## License

Unless otherwise specified, the code of this project are open source under the
GNU General Public License version 3 or any later version, while documentation,
configuration files, and scripts used in the development and maintenance process
are open source under the MIT License.

This project complies with the [REUSE specification].

You can use [reuse-tool](https://github.com/fsfe/reuse-tool) to generate the
SPDX list for this project:

```bash
reuse spdx
```

[REUSE specification]: https://reuse.software/spec-3.3/
