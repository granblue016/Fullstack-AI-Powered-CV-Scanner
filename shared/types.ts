export interface AnalysisVerdict {
  overall_score: number;        // 0-100
  verdict: "Absolutely Get Hired" | "Maybe or maybe not" | "Absolutely Get Rejected";
  matching_skills: string[];
  missing_skills: string[];
  ghost_skills: string[];       // Implied but not stated
  red_flags: string[];          // Gaps, job hopping
  confidence_level: number;     // 0.0-1.0
  explanation_vietnamese?: {
    summary: string;
    strengths: string[];
    weaknesses: string[];
    score_breakdown: {
      technical_match: string;
      experience_level: string;
      culture_fit: string;
      growth_potential: string;
    };
    recommendation: string;
  };
}

// Job Description input - can be file or manual text
export interface JobDescriptionInput {
  type: "file" | "text";
  content?: string;           // For text input
  fileName?: string;          // For file upload
  fileContent?: Uint8Array;   // For file upload
}

// Generated cover letter
export interface GeneratedCoverLetter {
  id: string;
  candidate_name: string;
  company_name: string;
  position_title: string;
  content: string;  // HTML or plain text
  created_at: string;
  match_score: number;
  key_points: string[];  // Key points highlighted in cover letter
}

// Generated email reply
export interface GeneratedEmailReply {
  id: string;
  recipient_email?: string;
  subject: string;
  body: string;
  html_body?: string;
  tone: "professional" | "friendly" | "enthusiastic";
  created_at: string;
  draft: boolean;  // Is this a draft or ready to send
}

// Combined output from analysis + cover letter + email
export interface ApplicationPackage {
  analysis: AnalysisVerdict;
  cover_letter: GeneratedCoverLetter;
  email_reply: GeneratedEmailReply;
  jd_analysis: {
    company_name: string;
    position_title: string;
    min_experience_years?: number;
    required_skills: string[];
    nice_to_have_skills: string[];
    salary_range?: string;
  };
}

// Job listing from aggregation
export interface JobListing {
  id: string;
  source: "topdev" | "itviec" | "linkedin" | "workable" | "datalytix" | "remote_ok" | "remoteok" | "flexjobs" | "weworkremotely";
  title: string;
  company: string;
  location: string;
  remote_status: "onsite" | "hybrid" | "remote" | "mixed";
  experience_level: string;
  salary_range?: {
    min?: number;
    max?: number;
    currency: string;
  };
  description: string;
  required_skills: string[];
  benefits?: string[];
  posted_at: string;
  url: string;
  logo_url?: string;
  match_score?: number;  // Match score based on user's CV
}

// Job search request
export interface JobSearchRequest {
  keywords: string[];
  location?: string;
  remote_only?: boolean;
  min_experience?: number;
  max_experience?: number;
  salary_min?: number;
  salary_max?: number;
  sources?: string[];  // Specific sources to search
  limit?: number;
}

// Job search response
export interface JobSearchResponse {
  total_count: number;
  jobs: JobListing[];
  search_time_ms: number;
  last_updated: string;
}

// Export formats for documents
export enum ExportFormat {
  PDF = "pdf",
  DOCX = "docx",
  HTML = "html"
}

// Document export request
export interface DocumentExportRequest {
  type: "cover_letter" | "email";
  format: ExportFormat;
  content: string;
  filename: string;
}
