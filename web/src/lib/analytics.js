export const GA_MEASUREMENT_ID = 'G-WZEYQGS8PE';
export const STORAGE_KEY = 'xfina_analytics_level';

// Analytics Levels
export const LEVEL_OFF = 'Off';
export const LEVEL_ANONYMOUS = 'Anonymous Usage';

// Generate a one-time random client ID for the current session to ensure anonymity.
// Since we don't use cookies, this ID is never persisted across reloads.
const SESSION_CLIENT_ID = Math.floor(Math.random() * 2147483647) + '.' + Math.floor(Date.now() / 1000);

export function getStoredAnalyticsLevel() {
    return localStorage.getItem(STORAGE_KEY) || LEVEL_ANONYMOUS;
}

export function setStoredAnalyticsLevel(level) {
    localStorage.setItem(STORAGE_KEY, level);
}

export function updateAnalyticsState(level) {
    // We no longer inject gtag.js at all! 
    // This function is kept for compatibility if needed, but does nothing now.
}

export function trackParserEvent(parserType, success, parseTime, validationMetrics = null, appVersion = 'unknown') {
    const level = getStoredAnalyticsLevel();
    if (level !== LEVEL_ANONYMOUS) {
        return;
    }

    // We manually construct the exact bare-minimum payload for Google Analytics.
    // By bypassing gtag.js, we prevent GA from automatically collecting device,
    // browser, screen resolution, URL, and OS metadata.
    const params = new URLSearchParams({
        v: '2', // GA4 version
        tid: GA_MEASUREMENT_ID,
        cid: SESSION_CLIENT_ID, // Required, but randomized so it's fully anonymous
        en: 'parser_usage', // Event name
        'ep.app_version': appVersion,
        'ep.parser_type': parserType,
        'ep.success': success.toString(),
        'epn.parse_time_ms': parseTime.toString(),
    });
    
    if (validationMetrics) {
        for (const [key, value] of Object.entries(validationMetrics)) {
            // Use epn. prefix for numeric custom parameters
            params.append(`epn.${key}`, value.toString());
        }
    }

    fetch(`https://www.google-analytics.com/g/collect?${params.toString()}`, {
        method: 'POST',
        mode: 'no-cors' // Google Analytics endpoint doesn't return CORS headers for this, so we use no-cors
    }).catch(err => {
        console.warn("Failed to send anonymous usage data");
    });
}
