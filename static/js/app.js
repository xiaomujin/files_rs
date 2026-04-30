const API_BASE = '/api';
const THEME_STORAGE_KEY = 'files-theme';
const THEME_LIGHT = 'light';
const THEME_DARK = 'dark';

const state = {
    currentRenameId: null,
    currentRenameName: '',
    activeLoads: 0,
    activeUploads: 0,
    isRenaming: false,
    loadRequestId: 0,
    theme: document.documentElement.dataset.theme || THEME_LIGHT,
};

const elements = {
    fileCount: document.getElementById('fileCount'),
    uploadArea: document.getElementById('uploadArea'),
    fileInput: document.getElementById('fileInput'),
    pickFileButton: document.getElementById('pickFileButton'),
    refreshButton: document.getElementById('refreshButton'),
    themeToggleButton: document.getElementById('themeToggleButton'),
    progressBar: document.getElementById('progressBar'),
    progressFill: document.getElementById('progressFill'),
    progressText: document.getElementById('progressText'),
    loadingState: document.getElementById('loadingState'),
    emptyState: document.getElementById('emptyState'),
    fileList: document.getElementById('fileList'),
    toastStack: document.getElementById('toastStack'),
    renameModal: document.getElementById('renameModal'),
    newFileName: document.getElementById('newFileName'),
    renameError: document.getElementById('renameError'),
    confirmRenameButton: document.getElementById('confirmRenameButton'),
    fileCardTemplate: document.getElementById('fileCardTemplate'),
};

function showToast(message, type = 'info') {
    const toast = document.createElement('div');
    toast.className = `toast toast-${type}`;
    toast.textContent = message;
    elements.toastStack.appendChild(toast);

    setTimeout(() => {
        toast.remove();
    }, 3200);
}

function formatSize(bytes) {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
}

function formatDate(dateStr) {
    const date = new Date(dateStr);
    return date.toLocaleString('zh-CN', {
        year: 'numeric',
        month: '2-digit',
        day: '2-digit',
        hour: '2-digit',
        minute: '2-digit',
    });
}

function getPreferredTheme() {
    return window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches
        ? THEME_DARK
        : THEME_LIGHT;
}

function setLoading(isLoading) {
    elements.loadingState.hidden = !isLoading;
    if (isLoading) {
        elements.emptyState.hidden = true;
        elements.fileList.hidden = true;
    }
}

function setEmptyState(isEmpty) {
    elements.emptyState.hidden = !isEmpty;
    elements.fileList.hidden = isEmpty;
}

function setProgress(active, percent = 0, text = '准备接收文件...') {
    elements.progressBar.classList.toggle('active', active);
    elements.progressFill.style.width = `${percent}%`;
    elements.progressText.textContent = text;
}

function isUploadingFiles() {
    return state.activeUploads > 0;
}

function isLoadingFiles() {
    return state.activeLoads > 0;
}

function getRenameValidationMessage(value) {
    const newName = value.trim();

    if (!newName) {
        return '请输入文件名';
    }

    if (newName.includes('/') || newName.includes('\\') || newName.includes('..')) {
        return '文件名不能包含 /、\\ 或 ..';
    }

    if (state.currentRenameName && newName === state.currentRenameName) {
        return '请输入一个不同的文件名';
    }

    return '';
}

function setRenameValidation(message = '') {
    const hasError = Boolean(message);
    elements.renameError.hidden = !hasError;
    elements.renameError.textContent = message;
    elements.newFileName.classList.toggle('input-error', hasError);
    syncUiState();
}

function updateThemeToggle() {
    if (!elements.themeToggleButton) {
        return;
    }

    const isDark = state.theme === THEME_DARK;
    elements.themeToggleButton.textContent = isDark ? '浅色模式' : '深色模式';
    elements.themeToggleButton.setAttribute('aria-pressed', String(isDark));
    elements.themeToggleButton.setAttribute('aria-label', isDark ? '切换到浅色模式' : '切换到深色模式');
}

function applyTheme(theme, persist = false) {
    state.theme = theme === THEME_DARK ? THEME_DARK : THEME_LIGHT;
    document.documentElement.dataset.theme = state.theme;
    updateThemeToggle();

    if (!persist) {
        return;
    }

    try {
        localStorage.setItem(THEME_STORAGE_KEY, state.theme);
    } catch (_) {
        // ignore storage access errors
    }
}

function toggleTheme() {
    const nextTheme = state.theme === THEME_DARK ? THEME_LIGHT : THEME_DARK;
    applyTheme(nextTheme, true);
}

function initTheme() {
    const initialTheme = document.documentElement.dataset.theme || getPreferredTheme();
    applyTheme(initialTheme, false);
}

function syncUiState() {
    const uploadBusy = isUploadingFiles();
    const loadBusy = isLoadingFiles();
    const renameValidationMessage = getRenameValidationMessage(elements.newFileName.value);

    elements.pickFileButton.disabled = uploadBusy;
    elements.fileInput.disabled = uploadBusy;
    elements.refreshButton.disabled = uploadBusy || loadBusy;
    elements.confirmRenameButton.disabled = state.isRenaming || !state.currentRenameId || Boolean(renameValidationMessage);

    elements.pickFileButton.textContent = uploadBusy ? '上传中...' : '选择文件';
    elements.refreshButton.textContent = loadBusy ? '刷新中...' : '刷新列表';
    elements.confirmRenameButton.textContent = state.isRenaming ? '保存中...' : '保存名称';

    elements.uploadArea.classList.toggle('is-disabled', uploadBusy);
    elements.uploadArea.setAttribute('aria-disabled', uploadBusy ? 'true' : 'false');
    updateThemeToggle();
}

function escapeAttribute(value) {
    return String(value)
        .replaceAll('&', '&amp;')
        .replaceAll('"', '&quot;')
        .replaceAll("'", '&#39;')
        .replaceAll('<', '&lt;')
        .replaceAll('>', '&gt;');
}

function renderFileItem(file) {
    const fragment = elements.fileCardTemplate.content.cloneNode(true);
    const card = fragment.querySelector('.file-card');
    const nameNode = fragment.querySelector('[data-role="file-name"]');
    const sizeNode = fragment.querySelector('[data-role="file-size"]');
    const dateNode = fragment.querySelector('[data-role="file-date"]');

    nameNode.textContent = file.original_name;
    nameNode.title = file.original_name;
    sizeNode.textContent = formatSize(file.size);
    dateNode.textContent = formatDate(file.created_at);

    card.dataset.id = file.id;
    card.dataset.name = escapeAttribute(file.original_name);

    return fragment;
}

function renderFiles(files, total) {
    elements.fileList.innerHTML = '';
    elements.fileCount.textContent = `共 ${total} 个文件`;

    if (!files.length) {
        setEmptyState(true);
        return;
    }

    setEmptyState(false);
    const fragment = document.createDocumentFragment();
    files.forEach((file) => {
        fragment.appendChild(renderFileItem(file));
    });
    elements.fileList.appendChild(fragment);
    elements.fileList.hidden = false;
}

async function loadFiles() {
    const requestId = ++state.loadRequestId;
    state.activeLoads += 1;
    setLoading(true);
    syncUiState();

    try {
        const response = await fetch(`${API_BASE}/files`);
        const result = await response.json();
        if (requestId !== state.loadRequestId) {
            return;
        }

        if (result.code !== 0) {
            throw new Error(result.message || '请求失败，请稍后重试');
        }

        renderFiles(result.data?.files || [], result.data?.total || 0);
    } catch (error) {
        if (requestId !== state.loadRequestId) {
            return;
        }

        setEmptyState(true);
        showToast(`加载文件列表失败: ${error.message}`, 'error');
    } finally {
        state.activeLoads = Math.max(0, state.activeLoads - 1);
        setLoading(isLoadingFiles());
        syncUiState();
    }
}

function beginUpload(fileName) {
    state.activeUploads += 1;
    setProgress(true, 0, `正在上传 ${fileName}...`);
    syncUiState();
}

function finishUpload() {
    state.activeUploads = Math.max(0, state.activeUploads - 1);

    if (isUploadingFiles()) {
        setProgress(true, 100, `已完成一个文件，剩余 ${state.activeUploads} 个上传任务...`);
    } else {
        setProgress(false, 0, '准备接收文件...');
    }

    syncUiState();
}

function uploadFile(file) {
    beginUpload(file.name);

    const formData = new FormData();
    formData.append('file', file);

    try {
        const xhr = new XMLHttpRequest();

        xhr.upload.addEventListener('progress', (event) => {
            if (!event.lengthComputable) {
                return;
            }
            const percent = (event.loaded / event.total) * 100;
            setProgress(true, percent, `正在上传 ${file.name} · ${Math.round(percent)}%`);
        });

        xhr.addEventListener('load', async () => {
            try {
                const response = JSON.parse(xhr.responseText || '{}');
                if (response.code === 0) {
                    showToast(`文件”${file.name}”上传成功`, 'success');
                    await loadFiles();
                } else {
                    showToast(response.message || '上传失败', 'error');
                }
            } finally {
                finishUpload();
            }
        });

        xhr.addEventListener('error', () => {
            finishUpload();
            showToast('上传失败，网络错误', 'error');
        });

        xhr.open('POST', `${API_BASE}/upload`);
        xhr.send(formData);
    } catch (error) {
        finishUpload();
        showToast(`上传失败: ${error.message}`, 'error');
    }
}

async function downloadFile(id, fileName) {
    try {
        const response = await fetch(`${API_BASE}/download/${encodeURIComponent(id)}`);
        const contentType = response.headers.get('Content-Type') || '';

        if (contentType.includes('application/json')) {
            const error = await response.json();
            showToast(error.message || '下载失败', 'error');
            return;
        }

        const blob = await response.blob();
        const url = window.URL.createObjectURL(blob);
        const link = document.createElement('a');
        link.href = url;
        link.download = fileName;
        document.body.appendChild(link);
        link.click();
        document.body.removeChild(link);
        window.URL.revokeObjectURL(url);

        showToast(`文件”${fileName}”下载成功`, 'success');
    } catch (error) {
        showToast(`下载失败: ${error.message}`, 'error');
    }
}

async function deleteFile(id, fileName) {
    if (!window.confirm(`确定要删除文件“${fileName}”吗？\n此操作不可恢复。`)) {
        return;
    }

    try {
        const response = await fetch(`${API_BASE}/files/${encodeURIComponent(id)}`, {
            method: 'DELETE',
        });
        const data = await response.json();

        if (data.code === 0) {
            showToast(`文件”${fileName}”删除成功`, 'success');
            await loadFiles();
        } else {
            showToast(data.message || '删除失败', 'error');
        }
    } catch (error) {
        showToast(`删除失败: ${error.message}`, 'error');
    }
}

function openRenameModal(id, currentName) {
    state.currentRenameId = id;
    state.currentRenameName = currentName;
    elements.renameModal.hidden = false;
    elements.newFileName.value = currentName;
    setRenameValidation('');
    elements.newFileName.focus();
    elements.newFileName.select();
    syncUiState();
}

function closeRenameModal() {
    if (state.isRenaming) {
        return;
    }

    elements.renameModal.hidden = true;
    state.currentRenameId = null;
    state.currentRenameName = '';
    elements.newFileName.value = '';
    setRenameValidation('');
    syncUiState();
}

async function confirmRename() {
    const validationMessage = getRenameValidationMessage(elements.newFileName.value);
    if (validationMessage) {
        setRenameValidation(validationMessage);
        elements.newFileName.focus();
        return;
    }

    if (!state.currentRenameId) {
        showToast('文件ID无效', 'error');
        return;
    }

    const newName = elements.newFileName.value.trim();
    state.isRenaming = true;
    syncUiState();

    let renameSucceeded = false;

    try {
        const response = await fetch(`${API_BASE}/files/${encodeURIComponent(state.currentRenameId)}`, {
            method: 'PUT',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({ new_name: newName }),
        });
        const data = await response.json();

        if (data.code === 0) {
            renameSucceeded = true;
            showToast('文件名已更新', 'success');
        } else {
            if (data.code >= 200 && data.code < 300) {
                setRenameValidation(data.message || '文件名无效');
            }
            showToast(data.message || '重命名失败', 'error');
        }
    } catch (error) {
        showToast(`重命名失败: ${error.message}`, 'error');
    } finally {
        state.isRenaming = false;
        syncUiState();
    }

    if (!renameSucceeded) {
        return;
    }

    closeRenameModal();
    await loadFiles();
}

function handleFileActionClick(target) {
    const actionButton = target.closest('[data-action]');
    if (!actionButton) {
        return;
    }

    const card = actionButton.closest('.file-card');
    if (!card) {
        return;
    }

    const id = card.dataset.id;
    const name = card.querySelector('[data-role="file-name"]')?.textContent || '';
    const action = actionButton.dataset.action;

    if (action === 'download') {
        downloadFile(id, name);
    } else if (action === 'rename') {
        openRenameModal(id, name);
    } else if (action === 'delete') {
        deleteFile(id, name);
    }
}

function bindEvents() {
    elements.pickFileButton.addEventListener('click', () => {
        if (isUploadingFiles()) {
            return;
        }
        elements.fileInput.click();
    });

    elements.refreshButton.addEventListener('click', () => {
        if (isLoadingFiles() || isUploadingFiles()) {
            return;
        }
        loadFiles();
    });

    elements.themeToggleButton?.addEventListener('click', toggleTheme);

    elements.uploadArea.addEventListener('dragover', (event) => {
        event.preventDefault();
        if (isUploadingFiles()) {
            return;
        }
        elements.uploadArea.classList.add('drag-over');
    });

    elements.uploadArea.addEventListener('dragleave', () => {
        elements.uploadArea.classList.remove('drag-over');
    });

    elements.uploadArea.addEventListener('drop', (event) => {
        event.preventDefault();
        elements.uploadArea.classList.remove('drag-over');

        if (isUploadingFiles()) {
            showToast('当前仍有文件在上传，请稍后再试', 'info');
            return;
        }

        const files = Array.from(event.dataTransfer.files || []);
        files.forEach(uploadFile);
    });

    elements.fileInput.addEventListener('change', (event) => {
        const files = Array.from(event.target.files || []);
        files.forEach(uploadFile);
        elements.fileInput.value = '';
    });

    elements.fileList.addEventListener('click', (event) => {
        handleFileActionClick(event.target);
    });

    elements.confirmRenameButton.addEventListener('click', confirmRename);

    elements.renameModal.addEventListener('click', (event) => {
        const action = event.target.closest('[data-action]')?.dataset.action;
        if (action === 'close-modal') {
            closeRenameModal();
        }
    });

    elements.newFileName.addEventListener('input', () => {
        setRenameValidation(getRenameValidationMessage(elements.newFileName.value));
    });

    elements.newFileName.addEventListener('keydown', (event) => {
        if (event.key === 'Enter') {
            confirmRename();
        } else if (event.key === 'Escape') {
            closeRenameModal();
        }
    });

    document.addEventListener('keydown', (event) => {
        if (event.key === 'Escape' && !elements.renameModal.hidden) {
            closeRenameModal();
        }
    });
}

function init() {
    initTheme();
    syncUiState();
    bindEvents();
    loadFiles();
}

document.addEventListener('DOMContentLoaded', init);
