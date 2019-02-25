# Contributing to grim

Thanks for your interest in contributing to grim! This document was written to
make sure the codebase is in a consistent format and assist anyone who wishes to
contribute in the future.

`grim` was written in an attempt to provide something useful to the forensic
community. Contributions are therefore absolutely welcome! We just ask that you
try to follow these guidelines when contributing to ensure both that everyone
feels welcome, and that the code is consistent and easily readable.

## Code of Conduct

Please abide by our
[Code of Conduct](https://github.com/demize/grim/blob/master/CODE_OF_CONDUCT.md)
at all times when participating in the community.

## How to Contribute
### Bug Reports

One of the easiest ways to contribute is to use grim and report any issues you
find. 

Before opening a new issue for a bug, confirm you're using the latest version of
grim. If not, try to reproduce the issue on the latest version. If you still
encounter the issue in the latest version, we then ask that you check the open
issues and refrain from opening a new issue if one already exists describing 
your bug.

If you find an existing issue describing your bug, please add any additional
information you have to that issue. Additional information is always useful when
trying to track down a bug!

If there is no existing issue, please open a new one and follow the bug report
template. To write a good bug report, use a clear and descriptive title, and
describe in detail the issue you encountered, including clear and concise steps
to reproduce. If you're able to help debug and troubleshoot, please let us know,
as sometimes an issue may be hardware-specific.

### Feature Requests

If you regularly use grim and would like to request we add a new feature, that's
great too! Submitting a feature request is a lot like submitting a bug report:
before submitting one, you should make sure it hasn't already been implemented
and that nobody else has requested it. When submitting a feature request, please
follow the feature request template.

### Your First Contribution

If you're not sure where to begin, take a look at the 
[good first issue](https://github.com/demize/grim/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22)
tag. Issues tagged as a good first issue are a good introduction to grim's
codebase, and so they should be a good first issue for a newcomer.

### Pull Requests

When submitting a pull request, please make sure it follows the style guides
(discussed below), does not break any existing code, and does not have any major
warnings when compiled/linted.

When submitting a pull request, you *must* use the provided template. Your pull
request title should be a clear and concise description of the changes you made,
and should not directly reference any issues (by name or number). The template
has three sections for description: one describing the change generally, one for
the benefits of the change, and one for the drawbacks of the change. These
sections should be as descriptive as possible.

## Coding Style

Follow Rust conventions, and use rustfmt or rls to format your code prior to
committing it.

## Documentation Style

Rust documentation comments should be used; this document will be updated at a
later date to specify proper documentation structure.

## License and Attribution

This file is distributed under the MIT license. See LICENSE for details.

This file is based on portions of the [Atom Contribution Guide](https://github.com/atom/atom/blob/ca71d581036ed093dd2df964fcc9bec0b5f7ff0d/CONTRIBUTING.md).