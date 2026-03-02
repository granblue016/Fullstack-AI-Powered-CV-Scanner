import { useState, useCallback } from 'react';
import { Upload, FileText, Loader2 } from 'lucide-react';
import { useMutation } from '@tanstack/react-query';
import type { AnalysisVerdict } from '../../../shared/types';

interface UploadZoneProps {
  onAnalysisComplete: (result: AnalysisVerdict) => void;
}

export default function UploadZone({ onAnalysisComplete }: UploadZoneProps) {
  const [isDragging, setIsDragging] = useState(false);
  const [cvFile, setCvFile] = useState<File | null>(null);
  const [jdText, setJdText] = useState('');

  const analysisMutation = useMutation({
    mutationFn: async () => {
      const formData = new FormData();
      if (cvFile) formData.append('cv', cvFile);
      formData.append('jd', jdText);

      const apiUrl = import.meta.env.PUBLIC_API_URL || 'http://localhost:3001';
      const res = await fetch(`${apiUrl}/api/analyze`, {
        method: 'POST',
        body: formData,
      });
      
      if (!res.ok) {
        throw new Error('Analysis failed');
      }
      
      return res.json() as Promise<AnalysisVerdict>;
    },
    onSuccess: onAnalysisComplete,
  });

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);
    const file = e.dataTransfer.files[0];
    if (file && (file.type === 'application/pdf' || file.name.endsWith('.docx'))) {
      setCvFile(file);
    }
  }, []);

  const handleFileInput = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) {
      setCvFile(file);
    }
  };

  return (
    <div className="space-y-6">
      {/* CV Upload Zone */}
      <div
        onDragOver={(e) => { e.preventDefault(); setIsDragging(true); }}
        onDragLeave={() => setIsDragging(false)}
        onDrop={handleDrop}
        onClick={() => document.getElementById('file-input')?.click()}
        className={`
          relative h-64 rounded-2xl border-2 border-dashed transition-all duration-300
          ${isDragging 
            ? 'border-blue-500 bg-blue-500/10 scale-105' 
            : cvFile
            ? 'border-green-500/50 bg-green-500/5'
            : 'border-white/20 bg-white/5 animate-pulse-border'
          }
          backdrop-blur-lg flex items-center justify-center cursor-pointer
          hover:border-white/40 hover:bg-white/10
        `}
      >
        <input
          id="file-input"
          type="file"
          accept=".pdf,.docx"
          onChange={handleFileInput}
          className="hidden"
        />
        <div className="text-center space-y-4 pointer-events-none">
          {cvFile ? (
            <>
              <FileText className="w-16 h-16 mx-auto text-green-400" />
              <div>
                <p className="text-lg text-white font-medium">{cvFile.name}</p>
                <p className="text-sm text-gray-400">
                  {(cvFile.size / 1024).toFixed(1)} KB
                </p>
              </div>
            </>
          ) : (
            <>
              <Upload className="w-16 h-16 mx-auto text-gray-400" />
              <div>
                <p className="text-xl text-white">Drop your CV here</p>
                <p className="text-sm text-gray-400">PDF or DOCX • Max 10MB</p>
              </div>
            </>
          )}
        </div>
      </div>

      {/* JD Input */}
      <div className="glass-panel p-6">
        <label className="block text-sm font-medium text-gray-300 mb-3">
          Job Description
        </label>
        <textarea
          value={jdText}
          onChange={(e) => setJdText(e.target.value)}
          placeholder="Paste the job description here... (no word limit) 📋&#10;&#10;Example:&#10;We are looking for a Senior Software Engineer 💼&#10;• 5+ years Python experience 🐍&#10;• React & TypeScript proficiency ⚛️&#10;• Docker & Kubernetes knowledge 🐳&#10;• Excellent communication skills 💬&#10;• Team player 👥"
          rows={12}
          className="w-full bg-black/30 backdrop-blur-sm border border-white/10 
                     rounded-xl p-4 text-white placeholder:text-gray-500 
                     focus:outline-none focus:border-white/30 focus:ring-2 focus:ring-blue-500/20
                     resize-vertical overflow-auto"
        />
      </div>

      {/* Analyze Button */}
      <button
        onClick={() => analysisMutation.mutate()}
        disabled={!cvFile || !jdText.trim() || analysisMutation.isPending}
        className="w-full py-4 bg-gradient-to-r from-blue-600 to-purple-600 
                   rounded-2xl font-semibold text-white text-lg
                   disabled:opacity-50 disabled:cursor-not-allowed
                   hover:scale-105 hover:shadow-2xl hover:shadow-blue-500/50
                   active:scale-95
                   transition-all duration-200
                   flex items-center justify-center gap-3"
      >
        {analysisMutation.isPending ? (
          <>
            <Loader2 className="animate-spin" size={24} />
            <span>Đang chấm điểm CV...</span>
          </>
        ) : (
          <>
            <span>🤖 Chấm điểm CV theo keyword JD</span>
          </>
        )}
      </button>

      {analysisMutation.isError && (
        <div className="glass-panel p-4 border-red-500/50 bg-red-500/10">
          <p className="text-red-400 text-sm">
            ⚠️ Analysis failed. Make sure the backend is running on port 3001.
          </p>
        </div>
      )}
    </div>
  );
}
