import { useState } from 'react';
import UploadZone from './UploadZone';
import ResultsPanel from './ResultsPanel';
import EmailWriter from './EmailWriter.tsx';
import CoverLetterWriter from './CoverLetterWriter.tsx';
import type { AnalysisVerdict } from '../../../shared/types';

export default function App() {
  const [results, setResults] = useState<AnalysisVerdict | null>(null);
  const [activeTab, setActiveTab] = useState<'home' | 'email' | 'cover'>('home');

  const navButtonClass = (tab: 'home' | 'email' | 'cover') => `
    px-4 py-2 rounded-xl text-sm font-medium transition-all duration-200
    ${activeTab === tab
      ? 'bg-white/15 text-white border border-white/20'
      : 'text-gray-400 hover:text-white hover:bg-white/10 border border-transparent'}
  `;

  return (
    <div className="space-y-8">
      <nav className="glass-panel p-2 flex items-center gap-2 w-fit mx-auto">
        <button
          onClick={() => setActiveTab('home')}
          className={navButtonClass('home')}
        >
          Chấm điểm CV
        </button>
        <button
          onClick={() => setActiveTab('email')}
          className={navButtonClass('email')}
        >
          Viết mail
        </button>
        <button
          onClick={() => setActiveTab('cover')}
          className={navButtonClass('cover')}
        >
          Viết cover letter
        </button>
      </nav>

      {activeTab === 'home' && (!results ? (
        <div className="animate-in fade-in duration-500">
          <UploadZone onAnalysisComplete={setResults} />
        </div>
      ) : (
        <div className="space-y-6">
          <ResultsPanel data={results} />
          <button
            onClick={() => setResults(null)}
            className="w-full py-3 glass-panel glass-hover
                       text-white font-medium transition-all duration-200
                       hover:scale-105 active:scale-95"
          >
            ← Chấm điểm CV khác
          </button>
        </div>
      ))}

      {activeTab === 'email' && <EmailWriter />}

      {activeTab === 'cover' && <CoverLetterWriter />}
    </div>
  );
}
