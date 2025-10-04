import React from "react";
import ReactDOM from 'react-dom/client'
import { createBrowserRouter, RouterProvider } from 'react-router-dom'
import DownloadPage from './pages/DownloadPage/DownloadPage'
import AdminPage from './pages/AdminPage/AdminPage'
import NotFound from './pages/NotFound/NotFound'
import './index.css'

const router = createBrowserRouter([
    { path: '/admin', element: <AdminPage /> },
    { path: '/:slug', element: <DownloadPage /> },   // <- root slug route
    { path: '/', element: <div>home</div> },         // optional landing
    { path: '*', element: <NotFound /> },            // 404 fallback
])

ReactDOM.createRoot(document.getElementById('root')).render(
    <React.StrictMode>
        <RouterProvider router={router} />
    </React.StrictMode>
)
