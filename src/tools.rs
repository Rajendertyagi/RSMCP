mod calculate_directory_size;
mod apply_patch;
mod create_directory;
mod diff_files;
mod directory_tree;
mod edit_block;
mod edit_file;
mod extract_images_from_pdf;
mod find_duplicate_files;
mod find_empty_directories;
mod get_file_info;
mod git_blame;
mod git_diff;
mod git_log;
mod git_show;
mod git_status;
mod grep;
mod head_file;
mod list_allowed_directories;
mod list_directory;
mod list_directory_with_sizes;
mod list_docx_parts;
mod list_excel_sheets;
mod list_pdf_pages;
mod list_tree;
mod move_file;
mod process_kill;
mod process_list;
mod process_ps;
mod process_read;
mod process_start;
mod process_write;
mod read_docx;
mod read_excel;
mod read_file_lines;
mod read_media_file;
mod read_multiple_media_files;
mod read_multiple_text_files;
mod read_pdf;
mod read_text_file;
mod search_and_replace;
mod search_file;
mod search_files_content;
mod tail_file;
mod write_excel;
mod write_file;
mod zip_unzip;

pub use calculate_directory_size::{CalculateDirectorySize, FileSizeOutputFormat};
pub use apply_patch::ApplyPatch;
pub use create_directory::CreateDirectory;
pub use diff_files::DiffFiles;
pub use directory_tree::DirectoryTree;
pub use edit_block::EditBlock;
pub use edit_file::{EditFile, EditOperation};
pub use extract_images_from_pdf::ExtractImagesFromPdf;
pub use find_duplicate_files::FindDuplicateFiles;
pub use find_empty_directories::FindEmptyDirectories;
pub use get_file_info::GetFileInfo;
pub use git_blame::GitBlame;
pub use git_diff::GitDiff;
pub use git_log::GitLog;
pub use git_show::GitShow;
pub use git_status::GitStatus;
pub use grep::Grep;
pub use head_file::HeadFile;
pub use list_allowed_directories::ListAllowedDirectories;
pub use list_directory::ListDirectory;
pub use list_directory_with_sizes::ListDirectoryWithSizes;
pub use list_docx_parts::ListDocxParts;
pub use list_excel_sheets::ListExcelSheets;
pub use list_pdf_pages::ListPdfPages;
pub use list_tree::ListTree;
pub use move_file::MoveFile;
pub use process_kill::ProcessKill;
pub use process_list::ProcessList;
pub use process_ps::ProcessPs;
pub use process_read::ProcessRead;
pub use process_start::ProcessStart;
pub use process_write::ProcessWrite;
pub use read_docx::ReadDocx;
pub use read_excel::ReadExcel;
pub use read_file_lines::ReadFileLines;
pub use read_media_file::ReadMediaFile;
pub use read_multiple_media_files::ReadMultipleMediaFiles;
pub use read_multiple_text_files::ReadMultipleTextFiles;
pub use read_pdf::ReadPdf;
pub use read_text_file::ReadTextFile;
pub use rust_mcp_sdk::tool_box;
pub use search_and_replace::SearchAndReplace;
pub use search_file::SearchFiles;
pub use search_files_content::SearchFilesContent;
pub use tail_file::TailFile;
pub use write_excel::WriteExcel;
pub use write_file::WriteFile;
pub use zip_unzip::{UnzipFile, ZipDirectory, ZipFiles};
//Generate FileSystemTools enum , tools() function, and TryFrom<CallToolRequestParams> trait implementation
tool_box!(
    FileSystemTools,
    [
        ReadTextFile,
        CreateDirectory,
        DirectoryTree,
        EditFile,
        GetFileInfo,
        ListAllowedDirectories,
        ListDirectory,
        MoveFile,
        ReadMultipleTextFiles,
        SearchFiles,
        WriteFile,
        ZipFiles,
        UnzipFile,
        ZipDirectory,
        SearchFilesContent,
        ListDirectoryWithSizes,
        ReadMediaFile,
        ReadMultipleMediaFiles,
        HeadFile,
        TailFile,
        ReadFileLines,
        FindEmptyDirectories,
        CalculateDirectorySize,
        FindDuplicateFiles,
        ReadPdf,
        ListPdfPages,
        ExtractImagesFromPdf,
        ReadExcel,
        ListExcelSheets,
        WriteExcel,
        ReadDocx,
        ListDocxParts,
        EditBlock,
        SearchAndReplace,
        ProcessStart,
        ProcessRead,
        ProcessWrite,
        ProcessKill,
        ProcessList,
        ProcessPs,
        ApplyPatch,
        ListTree,
        GitStatus,
        GitLog,
        GitShow,
        GitDiff,
        GitBlame,
        Grep,
        DiffFiles
    ]
);

impl FileSystemTools {
    // Determines whether the filesystem tool requires write access to the filesystem.
    // Returns `true` for tools that modify files or directories, and `false` otherwise.
    pub fn require_write_access(&self) -> bool {
        match self {
            FileSystemTools::CreateDirectory(_)
            | FileSystemTools::MoveFile(_)
            | FileSystemTools::WriteFile(_)
            | FileSystemTools::EditFile(_)
            | FileSystemTools::ZipFiles(_)
            | FileSystemTools::UnzipFile(_)
            | FileSystemTools::ZipDirectory(_)
            | FileSystemTools::WriteExcel(_)
            | FileSystemTools::EditBlock(_)
            | FileSystemTools::SearchAndReplace(_)
            | FileSystemTools::ProcessWrite(_)
            | FileSystemTools::ApplyPatch(_) => true,
            FileSystemTools::ReadTextFile(_)
            | FileSystemTools::DirectoryTree(_)
            | FileSystemTools::GetFileInfo(_)
            | FileSystemTools::ListAllowedDirectories(_)
            | FileSystemTools::ListDirectory(_)
            | FileSystemTools::ReadMultipleTextFiles(_)
            | FileSystemTools::SearchFilesContent(_)
            | FileSystemTools::ListDirectoryWithSizes(_)
            | FileSystemTools::ReadMediaFile(_)
            | FileSystemTools::HeadFile(_)
            | FileSystemTools::ReadMultipleMediaFiles(_)
            | FileSystemTools::TailFile(_)
            | FileSystemTools::ReadFileLines(_)
            | FileSystemTools::FindEmptyDirectories(_)
            | FileSystemTools::CalculateDirectorySize(_)
            | FileSystemTools::FindDuplicateFiles(_)
            | FileSystemTools::SearchFiles(_)
            | FileSystemTools::ReadPdf(_)
            | FileSystemTools::ListPdfPages(_)
            | FileSystemTools::ExtractImagesFromPdf(_)
            | FileSystemTools::ReadExcel(_)
            | FileSystemTools::ListExcelSheets(_)
            | FileSystemTools::ReadDocx(_)
            | FileSystemTools::ListDocxParts(_)
            | FileSystemTools::ProcessStart(_)
            | FileSystemTools::ProcessRead(_)
            | FileSystemTools::ProcessKill(_)
            | FileSystemTools::ProcessList(_)
            | FileSystemTools::ProcessPs(_)
            | FileSystemTools::ListTree(_)
            | FileSystemTools::GitStatus(_)
            | FileSystemTools::GitLog(_)
            | FileSystemTools::GitShow(_)
            | FileSystemTools::GitDiff(_)
            | FileSystemTools::GitBlame(_)
            | FileSystemTools::Grep(_)
            | FileSystemTools::DiffFiles(_) => false,
        }
    }
}
