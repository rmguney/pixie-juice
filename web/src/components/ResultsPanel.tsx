import { useEffect, useRef } from 'react';
import type { ProcessedSuccess, ResultsPanelProps } from '../types';
import { outputNames } from '../utils/files';
import { zip } from '../utils/zip';
import { formatBytes, formatChange } from '../utils/format';
import { outputFormat } from '../utils/outputFormat';
import { resultFile } from '../utils/resultFile';
import Icon from './Icon';

function save(blob: Blob, name: string) {
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = name;
  document.body.appendChild(link);
  link.click();
  link.remove();
  // Revoking synchronously can cancel the download in some browsers.
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}

/** Under `name`, unique within the batch (the file itself may be named alike to another). */
function download(result: ProcessedSuccess, name: string) {
  save(resultFile(result), name);
}

export default function ResultsPanel({ results, total, onReset }: ResultsPanelProps) {
  const successful = results.filter((r): r is ProcessedSuccess => r.success);
  const failed = results.length - successful.length;
  const names = new Map(
    outputNames(successful.map((r) => ({ name: r.file.name, extension: outputFormat(r.format).extension }))).map(
      (name, i) => [successful[i], name],
    ),
  );

  // One file as it is; several as one ZIP (browsers block or ask about many downloads at once).
  const downloadAll = () => {
    if (successful.length === 1) return download(successful[0], names.get(successful[0])!);
    save(zip(successful.map((result) => ({ name: names.get(result)!, data: result.data }))), 'pixie-juice.zip');
  };
  const notProcessed = total - results.length;

  // The panel replaces the one that had focus: move it here, so keyboard and
  // screen reader users land on the results.
  const heading = useRef<HTMLHeadingElement>(null);
  useEffect(() => {
    if (!document.activeElement || document.activeElement === document.body) heading.current?.focus();
  }, []);

  const before = successful.reduce((sum, r) => sum + r.file.size, 0);
  const after = successful.reduce((sum, r) => sum + r.data.length, 0);

  return (
    <section className="card panel" aria-labelledby="panel-title">
      <div className="card-head">
        <h2 className="card-title" id="panel-title" ref={heading} tabIndex={-1}>
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
            {notProcessed > 0 && <> · {notProcessed} not processed (cancelled)</>}
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
          {successful.length > 1 ? 'Download all (ZIP)' : 'Download'}
        </button>
        <button type="button" className="button" onClick={onReset}>
          Start over
        </button>
      </div>
    </section>
  );
}
