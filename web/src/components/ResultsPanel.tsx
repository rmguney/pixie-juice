import type { ResultsPanelProps, ProcessedResult } from '../types';
import { outputNames } from '../utils/files';
import { formatBytes, formatChange } from '../utils/format';
import { outputFormat } from '../utils/outputFormat';
import Icon from './Icon';

type Success = Extract<ProcessedResult, { success: true }>;

function download(result: Success, name: string) {
  const url = URL.createObjectURL(new Blob([result.data as BlobPart], { type: outputFormat(result.format).mimeType }));
  const link = document.createElement('a');
  link.href = url;
  link.download = name;
  document.body.appendChild(link);
  link.click();
  link.remove();
  // Revoking synchronously can cancel the download in some browsers.
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}

export default function ResultsPanel({ results, onReset }: ResultsPanelProps) {
  const successful = results.filter((r): r is Success => r.success);
  const failed = results.length - successful.length;
  const names = new Map(
    outputNames(successful.map((r) => ({ name: r.file.name, extension: outputFormat(r.format).extension }))).map(
      (name, i) => [successful[i], name],
    ),
  );

  // Staggered: browsers drop simultaneous programmatic downloads.
  const downloadAll = () =>
    successful.forEach((result, index) => setTimeout(() => download(result, names.get(result)!), index * 100));

  const before = successful.reduce((sum, r) => sum + r.file.size, 0);
  const after = successful.reduce((sum, r) => sum + r.data.length, 0);

  return (
    <section className="card panel" aria-labelledby="panel-title">
      <div className="card-head">
        <h2 className="card-title" id="panel-title">
          Results
        </h2>
      </div>

      {successful.length > 0 && (
        <div className="summary">
          <p className={`summary-figure mono${after < before ? ' juice-text' : ''}`}>{formatChange(before, after)}</p>
          <p className="summary-line">
            <span className="mono">
              {formatBytes(before)} → {formatBytes(after)}
            </span>{' '}
            · {successful.length} file{successful.length === 1 ? '' : 's'} processed
            {failed > 0 && <span className="error-text"> · {failed} failed</span>}
          </p>
        </div>
      )}

      <ul className="list" aria-label="Processed files">
        {results.map((result) => (
          <li key={result.file.name} className="row">
            <div className="row-main">
              <span className="row-text">
                <span className="row-name">{result.file.name}</span>
                {result.success ? (
                  <span className="row-meta">
                    <span>
                      {formatBytes(result.file.size)} → {formatBytes(result.data.length)}
                    </span>
                    {result.changed ? (
                      <span className={`pill${result.data.length < result.file.size ? ' good' : ''}`}>
                        {formatChange(result.file.size, result.data.length)}
                      </span>
                    ) : (
                      <span className="pill" title="Nothing smaller passed verification, so the original is kept">
                        already optimal
                      </span>
                    )}
                    <span className="pill">{outputFormat(result.format).extension.toUpperCase()}</span>
                  </span>
                ) : (
                  <span className="row-meta error-text">Failed: {result.error}</span>
                )}
              </span>
            </div>
            {result.success && (
              <button
                type="button"
                className="icon-button"
                onClick={() => download(result, names.get(result)!)}
                aria-label={`Download ${result.file.name}`}
              >
                <Icon name="download" />
              </button>
            )}
          </li>
        ))}
      </ul>

      <div className="actions">
        <button type="button" className="button primary" onClick={downloadAll} disabled={successful.length === 0}>
          <Icon name="download" />
          Download all
        </button>
        <button type="button" className="button" onClick={onReset}>
          Start over
        </button>
      </div>
    </section>
  );
}
