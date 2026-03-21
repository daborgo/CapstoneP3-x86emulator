import { NavLink } from 'react-router-dom'
import { LAB_COUNT } from './labConfig'

export default function Sidebar() {
  return (
    <nav className="nav-sidebar">
      <div className="nav-brand">
        <div className="nav-brand-asu">ASU</div>
        <div className="nav-brand-sub">x86 Emulator</div>
      </div>

      <div className="nav-section-label">LABS</div>

      <ul className="nav-list">
        {Array.from({ length: LAB_COUNT }, (_, i) => i + 1).map((n) => (
          <li key={n}>
            <NavLink
              to={`/lab${n}`}
              className={({ isActive }) =>
                `nav-link${isActive ? ' nav-link--active' : ''}`
              }
            >
              <span className="nav-link-badge">{n}</span>
              <span className="nav-link-text">Lab {n}</span>
            </NavLink>
          </li>
        ))}
      </ul>

      <div className="nav-footer">
        <span>x86-32 Subset</span>
      </div>
    </nav>
  )
}
