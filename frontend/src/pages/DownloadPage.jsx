import { useEffect, useState } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import NotFound from './NotFound'

export default function DownloadPage() {
    const { slug } = useParams()
    const nav = useNavigate()

    const [info, setInfo] = useState(null)
    const [loading, setLoading] = useState(true)
    const [error, setError] = useState(null)
    const [password, setPassword] = useState('')

    useEffect(() => {
        ; (async () => {
            setLoading(true)
            setError(null)
            try {
                const res = await fetch(`/api/share/${slug}`)
                if (res.status === 404) {
                    setLoading(false) // Keep info null
                    return
                }
                if (!res.ok) {
                    setError('Server error')
                    setLoading(false)
                    return
                }
                const data = await res.json()
                setInfo(data)
            } catch (e) {
                setError(`Network error: ${e}`)
            } finally {
                setLoading(false)
            }
        })()
        return () => { }
    }, [slug, nav])

    async function handleDownload() {
        setError(null)
        try {
            if (info?.password_required) {
                const res = await fetch(`/api/share/${slug}/check`, {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ password }),
                })
                if (res.status === 401) {
                    setError('Wrong password')
                    return
                }
                if (!res.ok) {
                    setError('Password check failed')
                    return
                }
            }
            // kick off the file download
            window.location.href = `/api/download/${slug}`
        } catch (e) {
            setError(`Network error ${e}`)
        }
    }

    if (loading) return <div style={{ padding: 20 }}>Loading…</div>
    if (error) return <div style={{ padding: 20, color: 'crimson' }}>{error}</div>
    if (!info) return <NotFound />

    return (
        <div style={{ padding: 20, maxWidth: 640 }}>
            <h1>{info.file_name}</h1>
            <p>Size: {info.file_size} bytes</p>
            {info.max_downloads != null && <p>Max downloads: {info.max_downloads}</p>}
            {info.expires_at && <p>Expires: {info.expires_at}</p>}

            {info.password_required && (
                <div style={{ margin: '12px 0' }}>
                    <input
                        type="password"
                        placeholder="Password"
                        value={password}
                        onChange={e => setPassword(e.target.value)}
                    />
                </div>
            )}

            <button onClick={handleDownload}>Download</button>
        </div>
    )
}
