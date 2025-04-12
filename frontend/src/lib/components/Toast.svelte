<script lang="ts">
    import { onMount } from 'svelte';
    
    export let message: string;
    export let type: 'success' | 'error' | 'warning';
    export let duration: number = 3000;
    export let onClose: () => void;
    
    let visible = false;
    
    onMount(() => {
        // Make the toast visible after a small delay for animation
        setTimeout(() => {
            visible = true;
        }, 10);
        
        // Auto-close after duration
        setTimeout(() => {
            visible = false;
            // Wait for fade out animation before calling onClose
            setTimeout(onClose, 300);
        }, duration);
    });
</script>

<div class="toast-container position-fixed top-0 start-50 translate-middle-x p-3" style="z-index: 9999;">
    <div class="toast show {type} {visible ? 'visible' : ''}" role="alert" aria-live="assertive" aria-atomic="true">
        <div class="toast-body d-flex align-items-center">
            {#if type === 'success'}
                <i class="bi bi-check-circle-fill me-2"></i>
            {:else if type === 'error'}
                <i class="bi bi-exclamation-circle-fill me-2"></i>
            {:else if type === 'warning'}
                <i class="bi bi-exclamation-triangle-fill me-2"></i>
            {/if}
            {message}
            <button type="button" class="btn-close ms-auto" on:click={() => { visible = false; setTimeout(onClose, 300); }}></button>
        </div>
    </div>
</div>

<style>
    .toast-container {
        pointer-events: none;
    }
    
    .toast {
        pointer-events: auto;
        background-color: rgba(33, 37, 41, 0.95);
        color: white;
        border: none;
        border-radius: 8px;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
        opacity: 0;
        transform: translateY(-20px);
        transition: opacity 0.3s ease, transform 0.3s ease;
        margin-top: 1rem;
    }
    
    .toast.visible {
        opacity: 1;
        transform: translateY(0);
    }
    
    .toast.success .toast-body {
        border-left: 4px solid #10B981;
    }
    
    .toast.error .toast-body {
        border-left: 4px solid #dc3545;
    }
    
    .toast.warning .toast-body {
        border-left: 4px solid #ffc107;
    }
    
    .toast.success i {
        color: #10B981;
    }
    
    .toast.error i {
        color: #dc3545;
    }
    
    .toast.warning i {
        color: #ffc107;
    }
    
    .btn-close {
        filter: invert(1) grayscale(100%) brightness(200%);
    }
    
    .toast-body {
        padding: 1rem;
        font-size: 0.9rem;
    }
</style> 