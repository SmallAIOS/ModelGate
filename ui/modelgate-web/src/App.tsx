import { useEffect, useState } from 'react';

import { ToastProvider } from './components/Toaster';
import { ModelsScreen } from './screens/ModelsScreen';
import { OverviewScreen } from './screens/OverviewScreen';
import { RoutesScreen } from './screens/RoutesScreen';
import { TerminalScreen } from './screens/TerminalScreen';

// --- Hash router ---

type Tab = 'overview' | 'models' | 'routes' | 'terminal';

function parseTab(hash: string): Tab {
  const cleaned = hash.replace(/^#\/?/, '');
  if (cleaned === 'models' || cleaned === 'routes' || cleaned === 'terminal') {
    return cleaned;
  }
  return 'overview';
}

function useHashRoute(): [Tab, (t: Tab) => void] {
  const [tab, setTab] = useState<Tab>(() => parseTab(window.location.hash));
  useEffect(() => {
    const onChange = () => setTab(parseTab(window.location.hash));
    window.addEventListener('hashchange', onChange);
    return () => window.removeEventListener('hashchange', onChange);
  }, []);
  const navigate = (t: Tab) => {
    window.location.hash = `#/${t}`;
    setTab(t);
  };
  return [tab, navigate];
}

// --- Shell ---

function App() {
  const [tab, navigate] = useHashRoute();
  return (
    <ToastProvider>
      <div className="shell">
        <header className="shell__top">
          <strong>ModelGate</strong>
        </header>
        <nav className="shell__rail">
          {(['overview', 'models', 'routes', 'terminal'] as Tab[]).map((t) => (
            <button
              key={t}
              type="button"
              className={t === tab ? 'rail__item rail__item--active' : 'rail__item'}
              onClick={() => navigate(t)}
            >
              {labelFor(t)}
            </button>
          ))}
        </nav>
        <main className="shell__main">
          {tab === 'overview' && <OverviewScreen />}
          {tab === 'models' && <ModelsScreen />}
          {tab === 'routes' && <RoutesScreen />}
          {tab === 'terminal' && <TerminalScreen />}
        </main>
      </div>
    </ToastProvider>
  );
}

function labelFor(t: Tab): string {
  switch (t) {
    case 'overview':
      return 'Overview';
    case 'models':
      return 'Models';
    case 'routes':
      return 'Routes';
    case 'terminal':
      return 'Terminal';
  }
}

export default App;
