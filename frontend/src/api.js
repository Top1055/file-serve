export async function fetchPublicShare(slug) {
    const res = await fetch(`/api/share/${encodeURIComponent(slug)}`);
    if (res.status === 404) return null;
    if (!res.ok) throw new Error(`server error: ${res.status}`);
    return res.json();
}

export function buildDownloadUrl(slug, password) {
    const base = `/api/download/${encodeURIComponent(slug)}`;
    if (!password) return base;
    const params = new URLSearchParams({ password });
    return `${base}?${params.toString()}`;
}
