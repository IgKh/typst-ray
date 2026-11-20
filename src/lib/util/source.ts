// SPDX-FileCopyrightText: 2026 Igor Khanin
// SPDX-License-Identifier: AGPL-3.0-or-later
import { compressToEncodedURIComponent, decompressFromEncodedURIComponent } from "lz-string";

const SOURCE_KEY = "source";
const SHARING_QS_PARAM = "source";

export function getInitialSource() {
    const sharedParam = new URLSearchParams(location.search).get(SHARING_QS_PARAM);
    if (sharedParam !== null) {
        const sharedSource = decompressFromEncodedURIComponent(sharedParam);
        if (sharedSource) {
            return { source: sharedSource, shared: true };
        }
    }
    return { source: localStorage.getItem(SOURCE_KEY) ?? "", shared: false };
}

export function saveSource(source: string) {
    // TODO: Debounce?
    localStorage.setItem(SOURCE_KEY, source);
}

export function makeSharingUrl(source: string) {
    const url = new URL(location.href);
    url.search = "";
    url.hash = "";
    url.searchParams.set(SHARING_QS_PARAM, compressToEncodedURIComponent(source));
    return url.toString();
}

export function clearSharingParam() {
    const url = new URL(location.href);
    url.searchParams.delete(SHARING_QS_PARAM);
    history.replaceState(history.state, "", url);
}
