import { CheckCircle, XCircle, AlertCircle, TrendingUp, Ghost, MessageSquare } from 'lucide-react';
import type { AnalysisVerdict } from '../../../shared/types';

interface ResultsPanelProps {
  data: AnalysisVerdict;
}

export default function ResultsPanel({ data }: ResultsPanelProps) {
  const getScoreColor = (score: number) => {
    if (score >= 80) return {
      text: 'text-green-400',
      border: 'border-green-500/50',
      bg: 'bg-green-500/20',
      glow: 'shadow-green-500/50'
    };
    if (score >= 60) return {
      text: 'text-amber-400',
      border: 'border-amber-500/50',
      bg: 'bg-amber-500/20',
      glow: 'shadow-amber-500/50'
    };
    return {
      text: 'text-red-400',
      border: 'border-red-500/50',
      bg: 'bg-red-500/20',
      glow: 'shadow-red-500/50'
    };
  };

  const colors = getScoreColor(data.overall_score);

  return (
    <div className="space-y-6 animate-in fade-in duration-500">
      {/* Score Badge */}
      <div className="text-center">
        <div className={`
          inline-flex items-center gap-4 px-8 py-6 rounded-full 
          border-2 ${colors.border} ${colors.bg} ${colors.text}
          shadow-2xl ${colors.glow}
          transition-all duration-300 hover:scale-105
        `}>
          <div className="text-center">
            <div className="text-6xl font-bold">{data.overall_score}%</div>
            <div className="text-sm opacity-75 mt-1">Match Score</div>
          </div>
          <div className="h-16 w-px bg-white/20" />
          <div className="text-left">
            <div className="text-2xl font-semibold">{data.verdict}</div>
            <div className="text-sm opacity-75">Recommendation</div>
          </div>
        </div>
      </div>

      {/* Skills Grid */}
      <div className="grid md:grid-cols-2 gap-6">
        {/* Matching Skills */}
        {data.matching_skills.length > 0 && (
          <div className="glass-panel p-6 glass-hover">
            <div className="flex items-center gap-2 mb-4">
              <CheckCircle className="text-green-400" size={24} />
              <h3 className="text-lg font-semibold text-white">
                Matching Skills
              </h3>
              <span className="ml-auto text-sm text-gray-400">
                {data.matching_skills.length}
              </span>
            </div>
            <ul className="space-y-2">
              {data.matching_skills.map((skill, idx) => (
                <li 
                  key={idx} 
                  className="text-gray-300 flex items-center gap-2 animate-in slide-in-from-left"
                  style={{ animationDelay: `${idx * 50}ms` }}
                >
                  <span className="w-1.5 h-1.5 bg-green-400 rounded-full" />
                  {skill}
                </li>
              ))}
            </ul>
          </div>
        )}

        {/* Missing Skills */}
        {data.missing_skills.length > 0 && (
          <div className="glass-panel p-6 glass-hover">
            <div className="flex items-center gap-2 mb-4">
              <XCircle className="text-red-400" size={24} />
              <h3 className="text-lg font-semibold text-white">
                Missing Skills
              </h3>
              <span className="ml-auto text-sm text-gray-400">
                {data.missing_skills.length}
              </span>
            </div>
            <ul className="space-y-2">
              {data.missing_skills.map((skill, idx) => (
                <li 
                  key={idx} 
                  className="text-gray-300 flex items-center gap-2 animate-in slide-in-from-right"
                  style={{ animationDelay: `${idx * 50}ms` }}
                >
                  <span className="w-1.5 h-1.5 bg-red-400 rounded-full" />
                  {skill}
                </li>
              ))}
            </ul>
          </div>
        )}

        {/* Ghost Skills */}
        {data.ghost_skills.length > 0 && (
          <div className="glass-panel p-6 glass-hover">
            <div className="flex items-center gap-2 mb-4">
              <TrendingUp className="text-blue-400" size={24} />
              <h3 className="text-lg font-semibold text-white">
                Ghost Skills
              </h3>
              <span className="ml-auto text-sm text-gray-400">
                {data.ghost_skills.length}
              </span>
            </div>
            <p className="text-xs text-gray-400 mb-3">
              Skills implied by experience but not explicitly listed
            </p>
            <ul className="space-y-2">
              {data.ghost_skills.map((skill, idx) => (
                <li 
                  key={idx} 
                  className="text-gray-300 flex items-center gap-2 animate-in slide-in-from-left"
                  style={{ animationDelay: `${idx * 50}ms` }}
                >
                  <span className="w-1.5 h-1.5 bg-blue-400 rounded-full" />
                  {skill}
                </li>
              ))}
            </ul>
          </div>
        )}

        {/* Red Flags */}
        {data.red_flags.length > 0 && (
          <div className="glass-panel p-6 glass-hover border-amber-500/30">
            <div className="flex items-center gap-2 mb-4">
              <AlertCircle className="text-amber-400" size={24} />
              <h3 className="text-lg font-semibold text-white">
                Red Flags
              </h3>
              <span className="ml-auto text-sm text-gray-400">
                {data.red_flags.length}
              </span>
            </div>
            <ul className="space-y-2">
              {data.red_flags.map((flag, idx) => (
                <li 
                  key={idx} 
                  className="text-gray-300 flex items-center gap-2 animate-in slide-in-from-right"
                  style={{ animationDelay: `${idx * 50}ms` }}
                >
                  <span className="w-1.5 h-1.5 bg-amber-400 rounded-full" />
                  {flag}
                </li>
              ))}
            </ul>
          </div>
        )}
      </div>

      {/* Vietnamese Explanation Section */}
      {data.explanation_vietnamese && (
        <div className="glass-panel p-6 space-y-4">
          <h3 className="text-lg font-bold text-white mb-3 flex items-center gap-2">
            <MessageSquare className="w-5 h-5 text-purple-400" />
            Phân Tích Chi Tiết (Vietnamese AI Analysis)
          </h3>
          <p className="text-gray-300 mb-4">{data.explanation_vietnamese.summary}</p>
          
          {/* Score Breakdown */}
          <div className="grid grid-cols-1 md:grid-cols-2 gap-3 mb-4">
            <div className="glass-panel p-4 border border-blue-500/30">
              <div className="text-blue-400 font-semibold mb-1">Kỹ Năng Kỹ Thuật</div>
              <div className="text-gray-300 text-sm">{data.explanation_vietnamese.score_breakdown.technical_match}</div>
            </div>
            <div className="glass-panel p-4 border border-green-500/30">
              <div className="text-green-400 font-semibold mb-1">Mức Độ Kinh Nghiệm</div>
              <div className="text-gray-300 text-sm">{data.explanation_vietnamese.score_breakdown.experience_level}</div>
            </div>
            <div className="glass-panel p-4 border border-yellow-500/30">
              <div className="text-yellow-400 font-semibold mb-1">Phù Hợp Văn Hóa</div>
              <div className="text-gray-300 text-sm">{data.explanation_vietnamese.score_breakdown.culture_fit}</div>
            </div>
            <div className="glass-panel p-4 border border-purple-500/30">
              <div className="text-purple-400 font-semibold mb-1">Tiềm Năng Phát Triển</div>
              <div className="text-gray-300 text-sm">{data.explanation_vietnamese.score_breakdown.growth_potential}</div>
            </div>
          </div>

          {/* Strengths */}
          <div className="mb-4">
            <h4 className="text-green-400 font-bold mb-2 flex items-center gap-2">
              <CheckCircle className="w-4 h-4" />
              Điểm Mạnh (Strengths)
            </h4>
            <ul className="space-y-2">
              {data.explanation_vietnamese.strengths.map((strength, idx) => (
                <li key={idx} className="text-gray-300 text-sm pl-4 border-l-2 border-green-500/50 py-1">
                  {strength}
                </li>
              ))}
            </ul>
          </div>

          {/* Weaknesses */}
          <div className="mb-4">
            <h4 className="text-red-400 font-bold mb-2 flex items-center gap-2">
              <XCircle className="w-4 h-4" />
              Điểm Yếu (Weaknesses)
            </h4>
            <ul className="space-y-2">
              {data.explanation_vietnamese.weaknesses.map((weakness, idx) => (
                <li key={idx} className="text-gray-300 text-sm pl-4 border-l-2 border-red-500/50 py-1">
                  {weakness}
                </li>
              ))}
            </ul>
          </div>

          {/* Recommendation */}
          <div className="glass-panel p-4 bg-gradient-to-r from-purple-500/10 to-blue-500/10 border border-purple-500/30">
            <h4 className="text-purple-400 font-bold mb-2">Khuyến Nghị (Recommendation)</h4>
            <p className="text-white text-sm">{data.explanation_vietnamese.recommendation}</p>
          </div>
        </div>
      )}

      {/* Confidence Indicator */}
      <div className="glass-panel p-4 text-center">
        <div className="flex items-center justify-center gap-2 text-sm text-gray-400">
          <span>AI Confidence:</span>
          <span className="text-white font-semibold">
            {(data.confidence_level * 100).toFixed(1)}%
          </span>
          <div className="ml-2 flex-1 max-w-xs h-2 bg-white/10 rounded-full overflow-hidden">
            <div 
              className="h-full bg-gradient-to-r from-blue-500 to-purple-500 transition-all duration-1000"
              style={{ width: `${data.confidence_level * 100}%` }}
            />
          </div>
        </div>
      </div>
    </div>
  );
}
