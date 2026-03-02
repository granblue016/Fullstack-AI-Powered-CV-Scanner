import { useState } from 'react';
import { useMutation } from '@tanstack/react-query';
import { FileText, Loader2 } from 'lucide-react';

type EmailType = 'initial_application' | 'interview_followup' | 'offer_response' | 'negotiation' | 'decline';

interface EmailReplyResponse {
  subject: string;
  body: string;
  email_type: string;
  tone: string;
}

export default function EmailWriter() {
  const [cvFile, setCvFile] = useState<File | null>(null);
  const [jdText, setJdText] = useState('');
  const [candidateName, setCandidateName] = useState('');
  const [candidateEmail, setCandidateEmail] = useState('');
  const [companyName, setCompanyName] = useState('');
  const [positionTitle, setPositionTitle] = useState('');
  const [recipientEmail, setRecipientEmail] = useState('');
  const [recipientName, setRecipientName] = useState('');
  const [emailType, setEmailType] = useState<EmailType>('initial_application');

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
      formData.append('recipient_email', recipientEmail);
      formData.append('recipient_name', recipientName);
      formData.append('email_type', emailType);

      const apiUrl = import.meta.env.PUBLIC_API_URL || 'http://localhost:3001';
      const res = await fetch(`${apiUrl}/api/email-reply`, {
        method: 'POST',
        body: formData,
      });

      if (!res.ok) {
        throw new Error('Generate email failed');
      }

      return res.json() as Promise<EmailReplyResponse>;
    },
  });

  return (
    <div className="space-y-6 animate-in fade-in duration-500">
      <div className="glass-panel p-6 space-y-4">
        <h2 className="text-xl font-semibold text-white">Viết mail từ CV + JD</h2>
        <p className="text-sm text-gray-400">Hệ thống tự extract: họ tên, SĐT, email, personal site, địa chỉ (ứng viên) và thông tin HR/công ty từ JD.</p>

        <div>
          <label htmlFor="email-cv-file" className="block text-sm text-gray-300 mb-2">CV (PDF/DOCX)</label>
          <input
            id="email-cv-file"
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
          <input
            value={recipientName}
            onChange={(e) => setRecipientName(e.target.value)}
            placeholder="Tên người nhận (tuỳ chọn)"
            className="bg-black/30 border border-white/10 rounded-xl p-3 text-white"
          />
          <input
            value={recipientEmail}
            onChange={(e) => setRecipientEmail(e.target.value)}
            placeholder="Email người nhận (tuỳ chọn)"
            className="bg-black/30 border border-white/10 rounded-xl p-3 text-white"
          />
        </div>

        <div>
          <label htmlFor="email-type" className="block text-sm text-gray-300 mb-2">Loại email</label>
          <select
            id="email-type"
            title="Chọn loại email"
            aria-label="Chọn loại email"
            value={emailType}
            onChange={(e) => setEmailType(e.target.value as EmailType)}
            className="w-full bg-black/30 border border-white/10 rounded-xl p-3 text-white"
          >
            <option value="initial_application">Ứng tuyển ban đầu</option>
            <option value="interview_followup">Follow up sau phỏng vấn</option>
            <option value="offer_response">Phản hồi offer</option>
            <option value="negotiation">Thương lượng</option>
            <option value="decline">Từ chối</option>
          </select>
        </div>

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
          className="w-full py-3 bg-gradient-to-r from-blue-600 to-purple-600 rounded-xl text-white font-semibold disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
        >
          {mutation.isPending ? (
            <>
              <Loader2 className="animate-spin" size={18} /> Đang tạo email...
            </>
          ) : (
            <span>✉️ Tạo email</span>
          )}
        </button>
      </div>

      {mutation.isSuccess && (
        <div className="glass-panel p-6 space-y-4">
          <div>
            <p className="text-sm text-gray-400 mb-1">Subject</p>
            <p className="text-white font-medium">{mutation.data.subject}</p>
          </div>
          <div>
            <p className="text-sm text-gray-400 mb-1">Nội dung email</p>
            <pre className="whitespace-pre-wrap text-gray-100 bg-black/30 rounded-xl p-4 border border-white/10">
              {mutation.data.body}
            </pre>
          </div>
          <p className="text-xs text-gray-400">
            Type: {mutation.data.email_type} • Tone: {mutation.data.tone}
          </p>
        </div>
      )}

      {mutation.isError && (
        <div className="glass-panel p-4 border-red-500/50 bg-red-500/10">
          <p className="text-red-400 text-sm">⚠️ Không thể tạo email. Kiểm tra backend/API key rồi thử lại.</p>
        </div>
      )}
    </div>
  );
}
