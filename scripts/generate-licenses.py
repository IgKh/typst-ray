#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Igor Khanin
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# Generates the attribution file for a given build from Cargo and NPM data
#
import dataclasses
import hashlib
import html
import json
import pathlib
import re
import string
import subprocess

from typing import Dict, List

LICENSE_REGEX = re.compile(r'^(LICEN[SC]E|COPYING|NOTICE|UNLICENSE)', re.IGNORECASE)
WHITESPACE_REGEX = re.compile(r'\s+')

TEMPLATE = string.Template("""
<!doctype html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <link rel="icon" href="/favicon.svg" type="image/svg+xml" />
    <title>Licenses - TypstRay</title>
    <style>
        :root { color-scheme: light dark; font-family: system-ui, sans-serif; }
        body { max-width: 60rem; margin: 0 auto; padding: 1rem 1.25rem 4rem; line-height: 1.5; }
        ul { padding-left: 1.25rem; }
        a, a:visited { color: LinkText; }
        section { border-top: 1px solid #8884; margin-top: 1.5rem; }
        pre { white-space: pre-wrap; font-size: .8rem; }
    </style>
</head>
<body>
    <h1>Licenses and Attribution</h1>
    <p>
        TypstRay is free software, licensed under the GNU AGPL v3 or later. Its license text and complete
        source code are available at <a href="https://github.com/IgKh/typst-ray">github.com/IgKh/typst-ray</a>.
    </p>
    <p>
        It includes software components primarily from <a href="https://typst.app">Typst</a> (licensed under
        Apache-2.0) as well as the following projects:
    </p>
    $body
</body>
</html>
""")


@dataclasses.dataclass
class DependencyItem:
    name: str
    license: str
    homepage: str
    texts: List[str]


@dataclasses.dataclass
class LicenseGroup:
    text: str
    items: List[DependencyItem]


def collect_license_texts(package_dir: pathlib.Path) -> List[str]:
    result = []
    for f in package_dir.iterdir():
        if f.is_file() and LICENSE_REGEX.match(f.name):
            result.append(f.read_text())

    return result


def collect_rust_licenses() -> List[DependencyItem]:
    cmd = ["cargo", "metadata", "--format-version", "1", "--filter-platform", "wasm32-unknown-unknown"]
    ret = subprocess.run(cmd, capture_output=True, check=True)
    data = json.loads(ret.stdout)

    pkg_map = {}
    for pkg in data["packages"]:
        pkg_map[pkg["name"]] = pkg

    result = []
    pkgs = [dep["name"] for dep in pkg_map["typstray-wasm"]["dependencies"]]
    visited = set()

    while pkgs:
        name = pkgs.pop(0)
        if name in visited or name not in pkg_map:
            continue

        visited.add(name)
        pkg = pkg_map[name]
        pkgs.extend(dep["name"] for dep in pkg["dependencies"])

        if pkg.get("kind") is not None or name.startswith("typst") or name.startswith("comemo"):
            continue

        homepage = pkg.get("homepage")
        if not homepage:
            homepage = pkg.get("repository")
        if not homepage:
            homepage = f"https://crates.io/crates/{name}"

        package_dir = pathlib.Path(pkg["manifest_path"]).parent

        result.append(DependencyItem(
            name=name,
            license=pkg.get("license", "See license text"),
            homepage=homepage,
            texts=collect_license_texts(package_dir)
        ))

    return result


def collect_npm_licenses() -> List[DependencyItem]:
    with open("package-lock.json", "r") as f:
        lock = json.load(f)

    pkg_map = lock["packages"]

    result = []
    pkgs = [dep for dep in pkg_map[""]["dependencies"]]
    visited = set()

    while pkgs:
        name = pkgs.pop(0)
        if name in visited:
            continue

        visited.add(name)
        pkg = pkg_map["node_modules/" + name]
        pkgs.extend(dep for dep in pkg.get("dependencies", []))

        package_dir = pathlib.Path("node_modules/" + name)

        result.append(DependencyItem(
            name=name,
            license=pkg["license"],
            homepage=f"https://www.npmjs.com/package/{name}",
            texts=collect_license_texts(package_dir)
        ))

    return result


def collect_font_licenses() -> List[DependencyItem]:
    return [
        DependencyItem(
            name="DejaVu Sans Mono Font",
            license="See license text",
            homepage="https://dejavu-fonts.github.io/",
            texts=[
                pathlib.Path("public/fonts/LICENSE-DejaVu").read_text()
            ]
        ),
        DependencyItem(
            name="Libertinus Serif Font",
            license="See license text",
            homepage="https://github.com/alerque/libertinus",
            texts=[
                pathlib.Path("public/fonts/LICENSE-Libertinus").read_text()
            ]
        ),
        DependencyItem(
            name="New Computer Modern Fonts",
            license="See license text",
            homepage="https://ctan.org/pkg/newcomputermodern",
            texts=[
                pathlib.Path("public/fonts/LICENSE-NewCM").read_text()
            ]
        )
    ]


def format_package_html(pkg: DependencyItem) -> str:
    return ("<li>" +
        f"<a href=\"{html.escape(pkg.homepage)}\">{html.escape(pkg.name)}</a> <span>({html.escape(pkg.license)})</span>" +
        "</li>")


def format_group_html(group: LicenseGroup) -> str:
    group.items.sort(key=lambda p: p.name)
    items = "\n".join(map(format_package_html, group.items))

    return ("<section>\n" +
        f"<ul>\n{items}\n</ul>\n" +
        "<details>\n" +
        "<summary>License text(s)</summary>\n"
        f"<pre>{html.escape(group.text)}</pre>\n"
        "</section>")


def run():
    packages = []
    packages.extend(collect_rust_licenses())
    packages.extend(collect_npm_licenses())
    packages.extend(collect_font_licenses())

    # Group based on exact license text
    groups: Dict[str, LicenseGroup] = {}
    for pkg in packages:
        text = "\n\n~~~~~~~~~\n\n".join(pkg.texts) if pkg.texts else f"(No license file provided, licensed under {pkg.license}.)"
        key = WHITESPACE_REGEX.sub(" ", text)
        digest = hashlib.sha256(key.encode()).hexdigest()

        if digest in groups:
            groups[digest].items.append(pkg)
        else:
            groups[digest] = LicenseGroup(text=text, items=[pkg])

    groups: List[LicenseGroup] = list(groups.values())
    groups.sort(key=lambda g: len(g.items), reverse=True)

    group_html = "\n".join(map(format_group_html, groups))
    html = TEMPLATE.substitute(body=group_html)

    out_file = "public/licenses.html"
    with open(out_file, "w") as f:
        f.write(html)
        print("Generated", out_file, "from", len(packages), "packages")

run()
