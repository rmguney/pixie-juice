import { useEffect, useState } from 'react';
import { getPixie } from './client';
import type { BuildInfo, PixieApi } from './types';

declare global {
  interface Window {
    /** The engine, for the browser console and end-to-end tests. */
    pixieJuice?: PixieApi;
  }
}

interface PixieState {
  api: PixieApi | null;
  info: BuildInfo | null;
  error: string | null;
}

/** Starts the engine worker and reports when it is ready. */
export function usePixie(): PixieState {
  const [state, setState] = useState<PixieState>({ api: null, info: null, error: null });

  useEffect(() => {
    let mounted = true;
    const api = getPixie();
    api
      .info()
      .then((info) => {
        window.pixieJuice = api;
        if (mounted) setState({ api, info, error: null });
      })
      .catch((error: unknown) => {
        if (mounted) setState({ api: null, info: null, error: error instanceof Error ? error.message : String(error) });
      });
    return () => {
      mounted = false;
    };
  }, []);

  return state;
}
