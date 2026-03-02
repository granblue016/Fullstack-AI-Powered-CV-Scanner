import { useState } from 'react';
import { useMutation } from '@tanstack/react-query';
import { FileText, Loader2 } from 'lucide-react';

interface CoverLetterResponse {
  content: string;
  key_points: string[];
  estimated_match_score: number;
}

export default function CoverLetterWriter() {
  const [cvFile, setCvFile] = useState<File | null>(null);
  const [jdText, setJdText] = useState('');
  const [candidateName, setCandidateName] = useState('');
  const [candidateEmail, setCandidateEmail] = useState('');
  const [companyName, setCompanyName] = useState('');
  const [positionTitle, setPositionTitle] = useState('');
  const [hiringManagerName, setHiringManagerName] = useState('');

  const mutation = useMutation({
    mutationFn: async () => {
      const formData = new FormData();
      if (cvFile) {
        formData.append('cv', cvFile);
      }
      formData.append('jd', jdText);
      formData.append('candidate_name', candidateName);
      formData.append('candidate_email', candidateEmail);
      formData.append('company_name', companyName);
      formData.append('position_title', positionTitle);
      formData.append('hiring_manager_name', hiringManagerName);

      const apiUrl = import.meta.env.PUBLIC_API_URL || 'http://localhost:3001';
      const res = await fetch(`${apiUrl}/api/cover-letter`, {
        method: 'POST',
        body: formData,
      });

      if (!res.ok) {
        throw new Error('Generate cover letter failed');
      }

      return res.json() as Promise<CoverLetterResponse>;
    },
  });

  return (
    <div className="space-y-6 animate-in fade-in duration-500">
      <div className="glass-panel p-6 space-y-4">
        <h2 className="text-xl font-semibold text-white">Viết cover letter từ CV + JD</h2>
        <p className="text-sm text-gray-400">Nếu bạn bỏ trống form, backend sẽ tự extract thông tin ứng viên/HR từ CV + JD để viết cover letter.</p>

        <div>
          <label htmlFor="cover-cv-file" className="block text-sm text-gray-300 mb-2">CV (PDF/DOCX)</label>
          <input
            id="cover-cv-file"
            type="file"
            accept=".pdf,.docx"
            title="Chọn file CV"
            aria-label="Chọn file CV"
            onChange={(e) => setCvFile(e.target.files?.[0] ?? null)}
            className="w-full bg-black/30 border border-white/10 rounded-xl p-3 text-gray-300"
          />
          {cvFile && (
            <p className="text-xs text-gray-400 mt-2 flex items-center gap-2">
              <FileText size={14} /> {cvFile.name}
            </p>
          )}
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <input
            value={candidateName}
            onChange={(e) => setCandidateName(e.target.value)}
            placeholder="Tên ứng viên"
            className="bg-black/30 border border-white/10 rounded-xl p-3 text-white"
          />
          <input
            value={candidateEmail}
            onChange={(e) => setCandidateEmail(e.target.value)}
            placeholder="Email ứng viên"
            className="bg-black/30 border border-white/10 rounded-xl p-3 text-white"
          />
          <input
            value={companyName}
            onChange={(e) => setCompanyName(e.target.value)}
            placeholder="Tên công ty"
            className="bg-black/30 border border-white/10 rounded-xl p-3 text-white"
          />
          <input
            value={positionTitle}
            onChange={(e) => setPositionTitle(e.target.value)}
            placeholder="Vị trí ứng tuyển"
            className="bg-black/30 border border-white/10 rounded-xl p-3 text-white"
          />
        </div>

        <input
          value={hiringManagerName}
          onChange={(e) => setHiringManagerName(e.target.value)}
          placeholder="Tên hiring manager (tuỳ chọn)"
          className="w-full bg-black/30 border border-white/10 rounded-xl p-3 text-white"
        />

        <textarea
          value={jdText}
          onChange={(e) => setJdText(e.target.value)}
          placeholder="Dán Job Description..."
          rows={10}
          className="w-full bg-black/30 border border-white/10 rounded-xl p-4 text-white placeholder:text-gray-500 resize-vertical"
        />

        <button
          onClick={() => mutation.mutate()}
          disabled={!cvFile || !jdText.trim() || mutation.isPending}
          className="w-full py-3 bg-gradient-to-r from-purple-600 to-pink-600 rounded-xl text-white font-semibold disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
        >
          {mutation.isPending ? (
            <>
              <Loader2 className="animate-spin" size={18} /> Đang tạo cover letter...
            </>
          ) : (
            <span>📝 Tạo cover letter</span>
          )}
        </button>
      </div>

      {mutation.isSuccess && (
        <div className="glass-panel p-6 space-y-4">
          <div className="flex items-center justify-between text-sm text-gray-300">
            <span>Estimated match</span>
            <span className="font-semibold text-white">{mutation.data.estimated_match_score}%</span>
          </div>

          {mutation.data.key_points.length > 0 && (
            <div>
              <p className="text-sm text-gray-400 mb-2">Key points</p>
              <ul className="list-disc list-inside text-gray-200 space-y-1">
                {mutation.data.key_points.map((point, idx) => (
                  <li key={idx}>{point}</li>
                ))}
              </ul>
            </div>
          )}

          <div>
            <p className="text-sm text-gray-400 mb-1">Nội dung cover letter</p>
            <pre className="whitespace-pre-wrap text-gray-100 bg-black/30 rounded-xl p-4 border border-white/10">
              {mutation.data.content}
            </pre>
          </div>
        </div>
      )}

      {mutation.isError && (
        <div className="glass-panel p-4 border-red-500/50 bg-red-500/10">
          <p className="text-red-400 text-sm">⚠️ Không thể tạo cover letter. Kiểm tra backend/API key rồi thử lại.</p>
        </div>
      )}
    </div>
  );
}
