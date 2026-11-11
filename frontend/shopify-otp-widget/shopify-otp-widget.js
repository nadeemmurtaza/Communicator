/**
 * Communicator OTP Widget for Shopify Stores
 *
 * This lightweight JavaScript widget intercepts the Shopify store's account
 * creation form, sends customer data to the Communicator OTP engine, and
 * handles OTP verification before completing the account creation.
 *
 * Usage:
 * 1. Add this script to your Shopify theme's customer/account/register.liquid
 * 2. Configure with your store's public key and Communicator API URL
 * 3. The widget automatically intercepts the form and handles OTP flow
 *
 * Example:
 * <script src="https://your-communicator.com/assets/shopify-otp-widget.js"></script>
 * <script>
 *   CommunicatorOTP.init({
 *     publicKey: 'pk_shop_abc123_xyz789',
 *     apiUrl: 'https://your-communicator.com/api/v1',
 *     storeDomain: 'mystore.myshopify.com'
 *   });
 * </script>
 */

(function (window) {
  'use strict';

  // ---------------------------------------------------------------------------
  // Configuration
  // ---------------------------------------------------------------------------

  let config = {
    publicKey: null,
    apiUrl: null,
    storeDomain: null,
    theme: {
      primaryColor: '#008060',       // Shopify green by default
      fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
      borderRadius: '8px',
      fontSize: '14px'
    },
    messages: {
      sending: 'Sending verification code...',
      sent: 'Code sent! Check your {channel} for the verification code.',
      verifyPlaceholder: 'Enter the verification code',
      verifyButton: 'Verify & Create Account',
      resendButton: 'Resend Code',
      success: 'Account verified! Redirecting...',
      error: 'Something went wrong. Please try again.',
      expired: 'Code expired. Please request a new one.',
      invalidCode: 'Invalid code. Please try again.',
      tooManyAttempts: 'Too many attempts. Please request a new code.',
      loading: 'Loading...'
    }
  };

  // ---------------------------------------------------------------------------
  // State
  // ---------------------------------------------------------------------------

  let state = {
    pendingId: null,
    otpId: null,
    step: 'initial',  // 'initial', 'otp_sent', 'verifying', 'verified', 'error'
    resendTimer: null,
    resendCountdown: 0
  };

  // ---------------------------------------------------------------------------
  // API Calls
  // ---------------------------------------------------------------------------

  async function apiCall(endpoint, body) {
    const response = await fetch(`${config.apiUrl}${endpoint}`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(body)
    });

    const data = await response.json();

    if (!response.ok) {
      throw new Error(data.error || `API error: ${response.status}`);
    }

    return data;
  }

  /**
   * Initiate account creation — sends customer data to Communicator
   * and triggers OTP generation via the brand's preferred channel.
   */
  async function initiateAccountCreation(customerData) {
    const endpoint = `/shopify/${encodeURIComponent(config.storeDomain)}/account/create`;

    const body = {
      email: customerData.email,
      phone: customerData.phone || customerData.phoneNumber || '',
      first_name: customerData.firstName || '',
      last_name: customerData.lastName || '',
      metadata: {
        shopify_token: customerData.formir || customerData.token || '',
        additional_fields: customerData.additionalFields || {}
      }
    };

    const result = await apiCall(endpoint, body);

    if (result.success) {
      state.pendingId = result.pending_id;
      state.otpId = result.otp_id;
      state.step = 'otp_sent';
    }

    return result;
  }

  /**
   * Verify the OTP code entered by the customer.
   */
  async function verifyOtp(code) {
    const result = await apiCall('/shopify/account/verify', {
      pending_id: state.pendingId,
      code: code
    });

    if (result.verified) {
      state.step = 'verified';
    }

    return result;
  }

  /**
   * Resend OTP if the customer didn't receive it.
   */
  async function resendOtp() {
    const result = await apiCall('/shopify/account/resend', {
      pending_id: state.pendingId
    });

    if (result.success) {
      state.otpId = result.otp_id;
      startResendTimer();
    }

    return result;
  }

  // ---------------------------------------------------------------------------
  // UI Components
  // ---------------------------------------------------------------------------

  /**
   * Create the OTP input widget element.
   */
  function createOtpWidget() {
    // Remove existing widget if any
    const existing = document.getElementById('communicator-otp-widget');
    if (existing) existing.remove();

    const widget = document.createElement('div');
    widget.id = 'communicator-otp-widget';
    widget.innerHTML = `
      <div class="communicator-otp-container" style="
        font-family: ${config.theme.fontFamily};
        font-size: ${config.theme.fontSize};
        max-width: 400px;
        margin: 20px auto;
        padding: 24px;
        border: 1px solid #e0e0e0;
        border-radius: ${config.theme.borderRadius};
        background: #ffffff;
        box-shadow: 0 2px 8px rgba(0,0,0,0.08);
      ">
        <div class="communicator-otp-header" style="
          text-align: center;
          margin-bottom: 20px;
        ">
          <div style="
            width: 48px;
            height: 48px;
            background: ${config.theme.primaryColor};
            border-radius: 50%;
            display: flex;
            align-items: center;
            justify-content: center;
            margin: 0 auto 12px;
            color: white;
            font-size: 24px;
          ">🔒</div>
          <h3 style="margin: 0 0 8px; color: #1a1a1a; font-size: 18px;">
            Verify Your Account
          </h3>
          <p style="margin: 0; color: #666; font-size: 14px;">
            We've sent a verification code to your <strong id="communicator-otp-channel"></strong>
          </p>
        </div>

        <div class="communicator-otp-input-group" style="margin-bottom: 16px;">
          <input
            type="text"
            id="communicator-otp-input"
            placeholder="${config.messages.verifyPlaceholder}"
            maxlength="20"
            autocomplete="one-time-code"
            style="
              width: 100%;
              padding: 12px 16px;
              border: 2px solid #e0e0e0;
              border-radius: ${config.theme.borderRadius};
              font-size: 18px;
              text-align: center;
              letter-spacing: 4px;
              font-family: monospace;
              box-sizing: border-box;
              transition: border-color 0.2s;
            "
            onfocus="this.style.borderColor='${config.theme.primaryColor}'"
            onblur="this.style.borderColor='#e0e0e0'"
          />
        </div>

        <div id="communicator-otp-error" style="
          color: #d32f2f;
          font-size: 13px;
          text-align: center;
          margin-bottom: 12px;
          display: none;
        "></div>

        <button
          id="communicator-otp-verify-btn"
          type="button"
          style="
            width: 100%;
            padding: 12px 24px;
            background: ${config.theme.primaryColor};
            color: white;
            border: none;
            border-radius: ${config.theme.borderRadius};
            font-size: 16px;
            font-weight: 600;
            cursor: pointer;
            transition: opacity 0.2s;
          "
          onmouseover="this.style.opacity='0.9'"
          onmouseout="this.style.opacity='1'"
        >
          ${config.messages.verifyButton}
        </button>

        <div style="
          text-align: center;
          margin-top: 16px;
          font-size: 13px;
          color: #666;
        ">
          <span>Didn't receive the code?</span>
          <button
            id="communicator-otp-resend-btn"
            type="button"
            style="
              background: none;
              border: none;
              color: ${config.theme.primaryColor};
              cursor: pointer;
              font-weight: 600;
              padding: 0;
              margin-left: 4px;
              font-size: 13px;
            "
          >
            ${config.messages.resendButton}
          </button>
          <span id="communicator-otp-resend-timer" style="display: none; color: #999;"></span>
        </div>

        <div id="communicator-otp-loading" style="
          display: none;
          text-align: center;
          padding: 12px;
          color: #666;
        ">
          <div style="
            display: inline-block;
            width: 20px;
            height: 20px;
            border: 2px solid #e0e0e0;
            border-top-color: ${config.theme.primaryColor};
            border-radius: 50%;
            animation: communicator-spin 0.8s linear infinite;
          "></div>
          <span style="margin-left: 8px;">${config.messages.loading}</span>
        </div>

        <style>
          @keyframes communicator-spin {
            to { transform: rotate(360deg); }
          }
        </style>
      </div>
    `;

    return widget;
  }

  /**
   * Show the OTP widget after the form is submitted.
   */
  function showOtpWidget(channel) {
    const channelEl = document.getElementById('communicator-otp-channel');
    if (channelEl) {
      channelEl.textContent = channel === 'whatsapp' ? 'WhatsApp' :
                              channel === 'sms' ? 'SMS' :
                              channel === 'email' ? 'email' : channel;
    }

    // Focus the input
    setTimeout(() => {
      const input = document.getElementById('communicator-otp-input');
      if (input) input.focus();
    }, 100);

    // Start resend timer
    startResendTimer();
  }

  /**
   * Hide the OTP widget.
   */
  function hideOtpWidget() {
    const widget = document.getElementById('communicator-otp-widget');
    if (widget) widget.style.display = 'none';
  }

  /**
   * Show loading state.
   */
  function showLoading() {
    const loading = document.getElementById('communicator-otp-loading');
    const verifyBtn = document.getElementById('communicator-otp-verify-btn');
    const input = document.getElementById('communicator-otp-input');
    if (loading) loading.style.display = 'block';
    if (verifyBtn) verifyBtn.disabled = true;
    if (input) input.disabled = true;
  }

  /**
   * Hide loading state.
   */
  function hideLoading() {
    const loading = document.getElementById('communicator-otp-loading');
    const verifyBtn = document.getElementById('communicator-otp-verify-btn');
    const input = document.getElementById('communicator-otp-input');
    if (loading) loading.style.display = 'none';
    if (verifyBtn) verifyBtn.disabled = false;
    if (input) input.disabled = false;
  }

  /**
   * Show error message.
   */
  function showError(message) {
    const errorEl = document.getElementById('communicator-otp-error');
    if (errorEl) {
      errorEl.textContent = message;
      errorEl.style.display = 'block';
    }
  }

  /**
   * Clear error message.
   */
  function clearError() {
    const errorEl = document.getElementById('communicator-otp-error');
    if (errorEl) {
      errorEl.textContent = '';
      errorEl.style.display = 'none';
    }
  }

  /**
   * Start the resend cooldown timer (60 seconds).
   */
  function startResendTimer() {
    const resendBtn = document.getElementById('communicator-otp-resend-btn');
    const timerEl = document.getElementById('communicator-otp-resend-timer');

    if (state.resendTimer) {
      clearInterval(state.resendTimer);
    }

    state.resendCountdown = 60;

    if (resendBtn) resendBtn.disabled = true;
    if (timerEl) timerEl.style.display = 'inline';

    state.resendTimer = setInterval(() => {
      state.resendCountdown--;

      if (timerEl) {
        timerEl.textContent = `(${state.resendCountdown}s)`;
      }

      if (state.resendCountdown <= 0) {
        clearInterval(state.resendTimer);
        if (resendBtn) resendBtn.disabled = false;
        if (timerEl) timerEl.style.display = 'none';
      }
    }, 1000);
  }

  // ---------------------------------------------------------------------------
  // Event Handlers
  // ---------------------------------------------------------------------------

  /**
   * Handle OTP verification button click.
   */
  async function handleVerify() {
    const input = document.getElementById('communicator-otp-input');
    const code = input ? input.value.trim() : '';

    if (!code) {
      showError('Please enter the verification code.');
      return;
    }

    clearError();
    showLoading();

    try {
      const result = await verifyOtp(code);

      if (result.verified) {
        // Success — complete the Shopify account creation
        await completeShopifyAccount();
      } else {
        showError(result.error === 'expired' ? config.messages.expired :
                  result.error === 'max_attempts' ? config.messages.tooManyAttempts :
                  config.messages.invalidCode);
      }
    } catch (error) {
      showError(error.message || config.messages.error);
    } finally {
      hideLoading();
    }
  }

  /**
   * Handle resend OTP button click.
   */
  async function handleResend() {
    const resendBtn = document.getElementById('communicator-otp-resend-btn');
    if (resendBtn && resendBtn.disabled) return;

    clearError();
    showLoading();

    try {
      const result = await resendOtp();

      if (result.success) {
        const channel = result.channel || 'the original channel';
        const channelDisplay = channel === 'whatsapp' ? 'WhatsApp' :
                               channel === 'sms' ? 'SMS' :
                               channel === 'email' ? 'email' : channel;
        showError('');  // Clear any previous errors
        // Show a brief success message
        const errorEl = document.getElementById('communicator-otp-error');
        if (errorEl) {
          errorEl.style.color = config.theme.primaryColor;
          errorEl.textContent = `New code sent via ${channelDisplay}!`;
          errorEl.style.display = 'block';
          setTimeout(() => {
            errorEl.style.display = 'none';
            errorEl.style.color = '#d32f2f';
          }, 3000);
        }
      } else {
        showError(result.error || config.messages.error);
      }
    } catch (error) {
      showError(error.message || config.messages.error);
    } finally {
      hideLoading();
    }
  }

  /**
   * Complete the Shopify account creation after OTP verification.
   * This submits the original form data to Shopify's account creation endpoint.
   */
  async function completeShopifyAccount() {
    const verifyBtn = document.getElementById('communicator-otp-verify-btn');
    if (verifyBtn) {
      verifyBtn.textContent = '✓ ' + config.messages.success;
      verifyBtn.disabled = true;
    }

    // The Shopify account is now verified. The store can:
    // 1. Let Communicator auto-create the customer (if configured)
    // 2. Or manually create the customer via Shopify Admin API
    //
    // For the widget, we redirect to the account login page or show success.
    setTimeout(() => {
      // Redirect to account login or home page
      const accountPage = window.location.pathname.includes('/account') ?
        '/account/login' : '/';
      window.location.href = accountPage;
    }, 1500);
  }

  // ---------------------------------------------------------------------------
  // Form Interception
  // ---------------------------------------------------------------------------

  /**
   * Intercept the Shopify account registration form.
   */
  function interceptForm() {
    // Try to find the Shopify registration form
    const forms = document.querySelectorAll('form[action*="/account/register"], form#customer_register, form.new_customer');

    forms.forEach(form => {
      form.addEventListener('submit', async (event) => {
        event.preventDefault();

        // Collect form data
        const formData = new FormData(form);
        const customerData = {};

        for (let [key, value] of formData.entries()) {
          customerData[key] = value;
        }

        // Map common Shopify form field names
        customerData.email = customerData.email || customerData['customer[email]'] || '';
        customerData.firstName = customerData.firstName || customerData['customer[first_name]'] || '';
        customerData.lastName = customerData.lastName || customerData['customer[last_name]'] || '';
        customerData.phone = customerData.phone || customerData['customer[phone]'] || '';
        customerData.password = customerData.password || customerData['customer[password]'] || '';
        customerData.formir = customerData.formir || customerData['formir-creation'] || '';

        // Store the original form for later submission
        state.originalForm = form;
        state.originalFormData = customerData;

        // Show loading on the form
        const submitBtn = form.querySelector('button[type="submit"], input[type="submit"]');
        if (submitBtn) {
          submitBtn.disabled = true;
          const originalText = submitBtn.textContent || submitBtn.value;
          submitBtn.dataset.originalText = originalText;
          submitBtn.textContent = config.messages.sending;
        }

        try {
          const result = await initiateAccountCreation(customerData);

          if (result.success) {
            // Hide the original form
            form.style.display = 'none';

            // Insert and show the OTP widget
            const widget = createOtpWidget();
            form.parentNode.insertBefore(widget, form.nextSibling);
            showOtpWidget(result.channel);

            // Attach event listeners
            const verifyBtn = document.getElementById('communicator-otp-verify-btn');
            if (verifyBtn) verifyBtn.addEventListener('click', handleVerify);

            const resendBtn = document.getElementById('communicator-otp-resend-btn');
            if (resendBtn) resendBtn.addEventListener('click', handleResend);

            // Enter key on input
            const input = document.getElementById('communicator-otp-input');
            if (input) {
              input.addEventListener('keypress', (e) => {
                if (e.key === 'Enter') handleVerify();
              });
            }
          } else {
            // Restore submit button
            if (submitBtn) {
              submitBtn.disabled = false;
              submitBtn.textContent = submitBtn.dataset.originalText || 'Create Account';
            }

            showError(result.error || config.messages.error);
          }
        } catch (error) {
          // Restore submit button
          if (submitBtn) {
            submitBtn.disabled = false;
            submitBtn.textContent = submitBtn.dataset.originalText || 'Create Account';
          }

          console.error('Communicator OTP Widget error:', error);
          showError(error.message || config.messages.error);
        }
      });
    });
  }

  // ---------------------------------------------------------------------------
  // Public API
  // ---------------------------------------------------------------------------

  function init(options) {
    Object.assign(config, options);

    if (config.theme) {
      Object.assign(config.theme, config.theme);
    }
    if (config.messages) {
      Object.assign(config.messages, config.messages);
    }

    // Validate required config
    if (!config.publicKey || !config.apiUrl || !config.storeDomain) {
      console.error('Communicator OTP Widget: publicKey, apiUrl, and storeDomain are required.');
      return;
    }

    console.log('Communicator OTP Widget initialized for store:', config.storeDomain);

    // If DOM is already loaded, intercept immediately
    if (document.readyState === 'complete' || document.readyState === 'interactive') {
      interceptForm();
    } else {
      // Otherwise wait for DOM to load
      document.addEventListener('DOMContentLoaded', interceptForm);
    }
  }

  // Expose public API
  window.CommunicatorOTP = {
    init: init,
    // Manual API for programmatic use
    initiateAccountCreation: initiateAccountCreation,
    verifyOtp: verifyOtp,
    resendOtp: resendOtp,
    createOtpWidget: createOtpWidget,
    getConfig: () => ({ ...config })
  };

})(window);
