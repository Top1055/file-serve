export async function getShare(slug) {
    const res = await fetch(`/api/share/${slug}`);
    return res.json();
}

export async function startDownload(slug, password = "") {
    const res = await fetch(`/api/download/${slug}`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ password })
    });
    return res;
}
