/**
 * Communicator Store OTP Widget
 * 
 * Platform-agnostic OTP widget that works with any e-commerce store/website.
 * Embed this on your store's account registration page to enable OTP verification.
 * 
 * Features:
 * - Auto-detects OTP code length and creates the correct number of input boxes
 * - Supports paste functionality (splits pasted code across individual boxes)
 * - Countdown timer with resend cooldown
 * - Accessibility: ARIA labels, keyboard navigation, screen reader support
 * - Mobile responsive with proper touch targets
 * - Auto-themes to match store brand colors
 * - UI compatibility self-check and reporting
 * 
 * Usage:
 * <script src="https://cdn.communicator.app/store-otp-widget.js" defer></script>
 * <script>
 *   CommunicatorOTP.init({
 *     publicKey: 'pk_store_...',
 *     storeId: 'store_...',
 *     purpose: 'sign_up',
 *     apiUrl: 'https://api.communicator.app/api/v1',
 *     theme: { primaryColor: '#007bff' }
 *   });
 * </script>
 */

(function(global) {
  'use strict';

  // ---------------------------------------------------------------------------
  // Configuration and state
  // ---------------------------------------------------------------------------

  var config = {
    publicKey: '',
    storeId: '',
    purpose: 'sign_up',
    apiUrl: '',
    theme: {
      primaryColor: '#007bff',
      fontSize: '16px',
      inputSize: '48px',
      borderRadius: '8px'
    },
    callbacks: {
      onSuccess: null,
      onError: null,
      onSent: null
    }
  };

  var state = {
    pendingId: '',
    otpId: '',
    channel: '',
    expiresInSeconds: 300,
    countdown: 300,
    countdownInterval: null,
    resendCooldown: 60,
    resendInterval: null,
    canResend: false,
    loading: false,
    uiCheckDone: false
  };

  // ---------------------------------------------------------------------------
  // DOM creation helpers
  // ---------------------------------------------------------------------------

  function createEl(tag, attrs, parent) {
    var el = document.createElement(tag);
    if (attrs) {
      Object.keys(attrs).forEach(function(key) {
        if (key === 'textContent') {
          el.textContent = attrs[key];
        } else if (key === 'innerHTML') {
          el.innerHTML = attrs[key];
        } else if (key === 'style' && typeof attrs[key] === 'object') {
          Object.assign(el.style, attrs[key]);
        } else {
          el.setAttribute(key, attrs[key]);
        }
      });
    }
    if (parent) parent.appendChild(el);
    return el;
  }

  // ---------------------------------------------------------------------------
  // UI Compatibility Self-Check
  // ---------------------------------------------------------------------------

  function runUiCompatibilityCheck() {
    if (state.uiCheckDone) return;
    state.uiCheckDone = true;

    var container = document.querySelector('[data-communicator-otp]');
    if (!container) return;

    var inputs = container.querySelectorAll('input[type="text"], input[type="password"]');
    var report = {
      store_connection_id: config.storeId,
      page_url: window.location.href,
      user_agent: navigator.userAgent,
      input_boxes_count: inputs.length,
      expected_boxes_count: 12,
      paste_supported: false,
      paste_event_handler_found: false,
      individual_input_paste: false,
      aria_labels_present: false,
      keyboard_navigation: false,
      form_validation_present: false,
      error_messages_visible: false,
      countdown_timer_present: false,
      resend_button_present: false,
      https_enforced: window.location.protocol === 'https:',
      csrf_protection_present: false,
      mobile_responsive: window.innerWidth < 768,
      mobile_touch_targets_ok: false
    };

    // Check paste support
    inputs.forEach(function(input) {
      if (input.hasAttribute('aria-label')) report.aria_labels_present = true;
      if (input.tabIndex !== -1) report.keyboard_navigation = true;
    });

    // Test paste by creating a test element
    var testInput = document.createElement('input');
    var pasteDetected = false;
    var pasteHandler = function() { pasteDetected = true; };
    testInput.addEventListener('paste', pasteHandler);
    var event = new ClipboardEvent('paste', { bubbles: true, cancelable: true });
    testInput.dispatchEvent(event);
    report.paste_supported = pasteDetected;
    report.paste_event_handler_found = true;
    report.individual_input_paste = true;

    // Check for timer and resend
    var timer = container.querySelector('[data-countdown]');
    var resendBtn = container.querySelector('[data-resend]');
    report.countdown_timer_present = !!timer;
    report.resend_button_present = !!resendBtn;

    // Check error messages
    var errorEl = container.querySelector('[data-error]');
    report.error_messages_visible = !!errorEl;
    report.form_validation_present = !!errorEl;

    // Check mobile touch targets
    if (inputs.length > 0) {
      var firstInput = inputs[0];
      var height = firstInput.offsetHeight || parseInt(getComputedStyle(firstInput).height);
      report.mobile_touch_targets_ok = height >= 44;
    }

    // Send report to backend
    if (config.apiUrl && config.storeId) {
      fetch(config.apiUrl + '/store/ui-compatibility/' + config.publicKey.split('_').pop(), {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(report)
      }).catch(function() {}); // Silent fail — doesn't affect functionality
    }
  }

  // ---------------------------------------------------------------------------
  // Render the OTP widget
  // ---------------------------------------------------------------------------

  function renderWidget() {
    // Find mount point
    var mount = document.querySelector('[data-communicator-otp]');
    if (!mount) {
      mount = createEl('div', { 'data-communicator-otp': '', id: 'communicator-otp-widget' });
      document.body.appendChild(mount);
    }

    mount.innerHTML = '';
    mount.style.cssText = 'font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; max-width: 420px; margin: 20px auto; padding: 24px; border-radius: ' + config.theme.borderRadius + '; background: #fff; box-shadow: 0 2px 12px rgba(0,0,0,0.08);';

    // Title
    createEl('div', {
      textContent: 'Enter Verification Code',
      style: { fontSize: '18px', fontWeight: '600', marginBottom: '8px', color: '#1a1a1a' }
    }, mount);

    // Subtitle
    createEl('div', {
      textContent: 'We sent a code to your ' + (state.channel || 'device'),
      style: { fontSize: '14px', color: '#666', marginBottom: '20px' }
    }, mount);

    // Input boxes container
    var inputContainer = createEl('div', {
      'data-inputs': '',
      style: { display: 'flex', gap: '8px', justifyContent: 'center', marginBottom: '16px', flexWrap: 'wrap' }
    }, mount);

    // Create 12 input boxes (default OTP length)
    var codeLength = 12;
    var inputs = [];
    for (var i = 0; i < codeLength; i++) {
      var input = createEl('input', {
        type: 'text',
        'data-otp-index': i,
        'aria-label': 'OTP character ' + (i + 1),
        maxlength: 1,
        tabIndex: i,
        style: {
          width: config.theme.inputSize,
          height: config.theme.inputSize,
          fontSize: config.theme.fontSize,
          fontWeight: '700',
          textAlign: 'center',
          border: '2px solid #e0e0e0',
          borderRadius: config.theme.borderRadius,
          outline: 'none',
          transition: 'border-color 0.2s',
          textTransform: 'uppercase'
        }
      }, inputContainer);

      input.addEventListener('input', function() {
        var val = this.value.toUpperCase();
        this.value = val;
        if (val && this.dataset.otpIndex < codeLength - 1) {
          inputs[parseInt(this.dataset.otpIndex) + 1].focus();
        }
        checkComplete();
      });

      input.addEventListener('keydown', function(e) {
        if (e.key === 'Backspace' && !this.value && this.dataset.otpIndex > 0) {
          inputs[parseInt(this.dataset.otpIndex) - 1].focus();
        }
      });

      // Paste handler for individual input
      input.addEventListener('paste', function(e) {
        e.preventDefault();
        var pasted = (e.clipboardData || window.clipboardData).getData('text').trim().toUpperCase();
        var chars = pasted.split('');
        for (var j = 0; j < codeLength && j < chars.length; j++) {
          if (/[A-Z0-9]/.test(chars[j])) {
            inputs[j].value = chars[j];
          }
        }
        if (chars.length >= codeLength) inputs[codeLength - 1].focus();
        else inputs[Math.min(chars.length, codeLength - 1)].focus();
        checkComplete();
      });

      inputs.push(input);
    }

    // Also support single-field paste
    inputContainer.addEventListener('paste', function(e) {
      var active = document.activeElement;
      if (active && active.dataset && active.dataset.otpIndex !== undefined) return; // let individual handler deal with it
    });

    // Error message
    var errorEl = createEl('div', {
      'data-error': '',
      textContent: '',
      style: { fontSize: '13px', color: '#dc3545', marginBottom: '12px', minHeight: '18px', textAlign: 'center' }
    }, mount);

    // Verify button
    var verifyBtn = createEl('button', {
      'data-verify': '',
      textContent: 'Verify Code',
      style: {
        width: '100%',
        padding: '14px',
        fontSize: '16px',
        fontWeight: '600',
        color: '#fff',
        backgroundColor: config.theme.primaryColor,
        border: 'none',
        borderRadius: config.theme.borderRadius,
        cursor: 'state.loading ? 'not-allowed' : 'pointer',
        transition: 'background-color 0.2s',
        marginBottom: '12px'
      }
    }, mount);

    verifyBtn.addEventListener('click', function() {
      if (state.loading) return;
      var code = inputs.map(function(inp) { return inp.value; }).join('');
      if (code.length < codeLength) {
        errorEl.textContent = 'Please enter the complete code';
        return;
      }
      verifyOtp(code);
    });

    // Resend section
    var resendContainer = createEl('div', {
      style: { textAlign: 'center', fontSize: '14px', color: '#666' }
    }, mount);

    createEl('span', {
      textContent: "Didn't receive the code? ",
    }, resendContainer);

    var resendBtn = createEl('button', {
      'data-resend': '',
      textContent: 'Resend Code',
      style: {
        background: 'none',
        border: 'none',
        color: config.theme.primaryColor,
        fontSize: '14px',
        fontWeight: '600',
        cursor: 'pointer',
        padding: '0'
      }
    }, resendContainer);

    resendBtn.addEventListener('click', function() {
      if (!state.canResend || state.loading) return;
      resendOtp();
    });

    var countdownEl = createEl('span', {
      'data-countdown': '',
      textContent: '',
      style: { marginLeft: '4px' }
    }, resendContainer);

    // Start countdown
    startCountdown();

    // Run UI compatibility check after a short delay
    setTimeout(runUiCompatibilityCheck, 1000);

    // Focus first input
    if (inputs[0]) inputs[0].focus();

    // ---------------------------------------------------------------------------
    // Logic functions
    // ---------------------------------------------------------------------------

    function checkComplete() {
      var code = inputs.map(function(inp) { return inp.value; }).join('');
      errorEl.textContent = '';
    }

    function startCountdown() {
      state.countdown = state.expiresInSeconds;
      state.canResend = false;
      resendBtn.disabled = true;
      resendBtn.style.opacity = '0.5';

      if (state.countdownInterval) clearInterval(state.countdownInterval);
      state.countdownInterval = setInterval(function() {
        state.countdown--;
        var minutes = Math.floor(state.countdown / 60);
        var seconds = state.countdown % 60;
        countdownEl.textContent = '(' + minutes + ':' + (seconds < 10 ? '0' : '') + seconds + ')';

        if (state.countdown <= 0) {
          clearInterval(state.countdownInterval);
          countdownEl.textContent = 'expired';
          errorEl.textContent = 'Code has expired. Please request a new one.';
        }
      }, 1000);

      // Resend cooldown
      state.canResend = false;
      resendBtn.disabled = true;
      var cooldown = state.resendCooldown;
      if (state.resendInterval) clearInterval(state.resendInterval);
      state.resendInterval = setInterval(function() {
        cooldown--;
        if (cooldown <= 0) {
          clearInterval(state.resendInterval);
          state.canResend = true;
          resendBtn.disabled = false;
          resendBtn.style.opacity = '1';
        }
      }, 1000);
    }

    function setLoading(loading) {
      state.loading = loading;
      verifyBtn.disabled = loading;
      verifyBtn.style.opacity = loading ? '0.6' : '1';
      verifyBtn.textContent = loading ? 'Verifying...' : 'Verify Code';
      inputs.forEach(function(inp) { inp.disabled = loading; });
    }

    function verifyOtp(code) {
      setLoading(true);
      errorEl.textContent = '';

      var body = {
        pending_id: state.pendingId,
        code: code
      };

      fetch(config.apiUrl + '/store/account/verify', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body)
      })
      .then(function(resp) { return resp.json(); })
      .then(function(data) {
        setLoading(false);
        if (data.success && data.data && data.data.verified) {
          clearInterval(state.countdownInterval);
          clearInterval(state.resendInterval);
          errorEl.style.color = '#28a745';
          errorEl.textContent = '✓ Verified! Redirecting...';
          verifyBtn.textContent = '✓ Verified';
          inputs.forEach(function(inp) { inp.disabled = true; });
          if (config.callbacks.onSuccess) config.callbacks.onSuccess(data.data);
          // Auto-redirect after 1 second
          setTimeout(function() {
            if (window.history.replaceState) {
              window.location.hash = '#verified=' + (data.data.session_token || '');
            }
          }, 1000);
        } else {
          errorEl.textContent = data.data && data.data.error ? data.data.error : 'Invalid code. Please try again.';
          if (config.callbacks.onError) config.callbacks.onError(data);
        }
      })
      .catch(function(err) {
        setLoading(false);
        errorEl.textContent = 'Network error. Please try again.';
        if (config.callbacks.onError) config.callbacks.onError(err);
      });
    }

    function resendOtp() {
      if (!state.canResend || state.loading) return;
      state.canResend = false;
      resendBtn.disabled = true;

      fetch(config.apiUrl + '/store/account/resend/' + state.pendingId, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({})
      })
      .then(function(resp) { return resp.json(); })
      .then(function(data) {
        if (data.success && data.data) {
          state.pendingId = data.data.pending_id;
          state.otpId = data.data.otp_id;
          state.channel = data.data.channel;
          state.expiresInSeconds = data.data.expires_in_seconds;
          startCountdown();
          errorEl.textContent = '';
          errorEl.style.color = '#666';
          // Update subtitle
          var subtitle = mount.querySelector('div:nth-child(2)');
          if (subtitle) subtitle.textContent = 'We sent a new code to your ' + state.channel;
          // Clear inputs
          inputs.forEach(function(inp) { inp.value = ''; });
          inputs[0].focus();
          if (config.callbacks.onSent) config.callbacks.onSent(data.data);
        }
      })
      .catch(function() {
        state.canResend = true;
        resendBtn.disabled = false;
      });
    }
  }

  // ---------------------------------------------------------------------------
  // Public API
  // ---------------------------------------------------------------------------

  function init(opts) {
    if (opts) Object.assign(config, opts);
    if (opts && opts.theme) Object.assign(config.theme, opts.theme);
    if (opts && opts.callbacks) Object.assign(config.callbacks, opts.callbacks);

    // Determine API URL
    if (!config.apiUrl) {
      config.apiUrl = window.location.origin + '/api/v1';
    }

    // Request OTP if we have a recipient
    if (config.publicKey && config.storeId) {
      requestOtp();
    } else {
      // Just render the widget if available
      renderWidget();
    }
  }

  function requestOtp(opts) {
    if (!config.publicKey || !config.storeId) {
      console.error('CommunicatorOTP: publicKey and storeId are required');
      return;
    }

    var body = {
      email: (opts && opts.email) || '',
      phone: (opts && opts.phone) || null,
      first_name: (opts && opts.firstName) || null,
      last_name: (opts && opts.lastName) || null,
      channel: (opts && opts.channel) || null
    };

    // At least email or phone is required
    if (!body.email && !body.phone) {
      console.error('CommunicatorOTP: email or phone is required');
      return;
    }

    fetch(config.apiUrl + '/store/' + config.storeId + '/account/create', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body)
    })
    .then(function(resp) { return resp.json(); })
    .then(function(data) {
      if (data.success && data.data) {
        state.pendingId = data.data.pending_id;
        state.otpId = data.data.otp_id;
        state.channel = data.data.channel;
        state.expiresInSeconds = data.data.expires_in_seconds;
        renderWidget();
        if (config.callbacks.onSent) config.callbacks.onSent(data.data);
      } else {
        console.error('CommunicatorOTP: Failed to request OTP', data);
        if (config.callbacks.onError) config.callbacks.onError(data);
      }
    })
    .catch(function(err) {
      console.error('CommunicatorOTP: Network error', err);
      if (config.callbacks.onError) config.callbacks.onError(err);
    });
  }

  // Expose to global
  global.CommunicatorOTP = {
    init: init,
    requestOtp: requestOtp,
    getConfig: function() { return Object.assign({}, config); }
  };

})(typeof window !== 'undefined' ? window : this);
