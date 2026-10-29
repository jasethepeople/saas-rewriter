import React from 'react';
import ReactDOM from 'react-dom/client';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { useAuthStore } from './hooks/useAuth';
import App from './components/App';
import AuthPage from './components/AuthPage';
import './index.css';

const queryClient = new QueryClient({
  defaultOptions: { queries: { retry: 1 } },
});

function Root() {
  const { isAuthenticated } = useAuthStore();
  return isAuthenticated ? <App /> : <AuthPage />;
}

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <QueryClientProvider client={queryClient}>
      <Root />
    </QueryClientProvider>
  </React.StrictMode>
);
