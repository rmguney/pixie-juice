import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import ResultsPanel from '../../src/components/ResultsPanel';
import type { ProcessedResult } from '../../src/types';

const file = (name: string, size: number) => new File([new Uint8Array(size)], name);

describe('ResultsPanel', () => {
  it('summarises savings, unchanged files and failures', () => {
    const results: ProcessedResult[] = [
      { file: file('a.png', 1000), success: true, data: new Uint8Array(400), format: 'png', changed: true },
      { file: file('b.jpg', 500), success: true, data: new Uint8Array(500), format: 'jpeg', changed: false },
      { file: file('c.obj', 10), success: false, error: 'Unknown file format' },
    ];
    render(<ResultsPanel results={results} total={3} onReset={vi.fn()} />);

    // 1500 B in, 900 B out across the two successes; a.png alone went from 1000 B to 400 B.
    expect(screen.getByText('−40%')).toBeInTheDocument();
    expect(screen.getByText(/2 files processed/)).toBeInTheDocument();
    expect(screen.getByText(/1 failed/)).toBeInTheDocument();
    expect(screen.getByText('−60%')).toBeInTheDocument();
    expect(screen.getByText('already optimal')).toBeInTheDocument();
    expect(screen.getByText('Failed: Unknown file format')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Download a.png' })).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Download c.obj' })).not.toBeInTheDocument();
  });
});
