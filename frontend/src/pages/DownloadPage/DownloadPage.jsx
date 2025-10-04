import { useEffect, useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { fetchPublicShare, buildDownloadUrl } from '../api';
import NotFound from './NotFound.jsx';

function formatBytes(n) {
    if (n == null) return '';
    const units = ['B', 'KB', 'MB', 'GB', 'TB'];
    let i = 0, x = n;
    while (x >= 1024 && i < units.length - 1) { x /= 1024; i++; }
    return `${x.toFixed(x < 10 && i > 0 ? 1 : 0)} ${units[i]}`;
}

export default function DownloadPage() {
    const { slug } = useParams();
    const nav = useNavigate();

    const [info, setInfo] = useState(null);
    const [password, setPassword] = useState('');
    const [loading, setLoading] = useState(true);
    const [err, setErr] = useState(null);

    useEffect(() => {
        (async () => {
            try {
                setLoading(true);
                setErr(null);
                const data = await fetchPublicShare(slug);
                if (!data) {
                    setInfo(null)
                    return;
                }
                console.log(data);
                setInfo(data);
            } catch (e) {
                setErr(`Server/network error: ${e}`);
            } finally {
                setLoading(false);
            }
        })();
        return () => { };
    }, [slug, nav]);

    function handleDownload() {
        // No separate password check needed: backend accepts ?password=
        const url = buildDownloadUrl(slug, info?.password_required ? password : undefined);
        // Navigate to the file URL; browser handles the download
        window.location.href = url;
    }

    if (loading) return <div style={{ padding: 20 }}>Loading…</div>;
    if (err) return <div style={{ padding: 20, color: 'crimson' }}>{err}</div>;
    if (!info) return <NotFound />;


    return (
        <div style={{ padding: 20, maxWidth: 720 }}>
            <h1>{info.file_name}</h1>
            <p>Size: {formatBytes(info.file_size)}</p>
            <p>Downloads: {info.dl_count}</p>
            {info.max_downloads != null && <p>Max downloads: {info.max_downloads}</p>}
            {info.expires_at && <p>Expires: {info.expires_at}</p>}

            {info.password_required && (
                <div style={{ margin: '12px 0' }}>
                    <label style={{ display: 'block', marginBottom: 6 }}>Password</label>
                    <input
                        type="password"
                        value={password}
                        onChange={e => setPassword(e.target.value)}
                        placeholder="••••••••"
                    />
                </div>
            )}

            <button onClick={handleDownload}>Download</button>
        </div>
    );
}
