import { fetchEventSource } from '@microsoft/fetch-event-source';
import axios from 'axios';

class ApiClient {
  init() {
    this._baseUrl =
      window.__BASE__ == '/AITER_BASE' || window.__BASE__ == '/' || !window.__BASE__
        ? '/api'
        : `${window.__BASE__}/api`;
    this._inited = true;
  }

  async get(url, params, options) {
    return this.request('GET', url, params, options);
  }

  async post(url, data, options) {
    return this.request('POST', url, data, options);
  }

  async postForm(url, form, options) {
    return this.request('POST', url, form, options);
  }

  async request(method, url, payload, options) {
    if (!this._inited) {
      throw new Error('API_NOT_INITED');
    }

    const axiosConfig = {
      baseURL: this._baseUrl,
      url,
      method,
    };

    {
      if (method.toLowerCase() === 'get') {
        axiosConfig.params = payload;
      } else if (method.toLowerCase() === 'post') {
        axiosConfig.data = payload;
      }

      axiosConfig.headers = options?.headers ?? {};
      const token = localStorage.getItem('aiter-token');
      if (token) {
        axiosConfig.headers['Authorization'] = `Bearer ${token}`;
      }

      const { abortCallback } = options ?? {};
      const abortController = new AbortController();
      if (abortCallback instanceof Function) {
        abortCallback(abortController);
      }
      axiosConfig.signal = abortController.signal;
    }

    try {
      const resp = await axios(axiosConfig);

      if (/^application\/json/i.test(resp.headers['content-type'])) {
        if (resp.data.error) {
          const e = new Error(resp.data.message);
          e.code = resp.data.error;
          throw e;
        }
      }

      return resp.data;
    } catch (err) {
      if (err.status == 401) {
        const e = new Error(err.message);
        e.code = 'UNAUTHORIZED';
        throw e;
      } else {
        throw err;
      }
    }
  }

  async sse(url, params, options) {
    if (!this._inited) {
      throw new Error('API_NOT_INITED');
    }

    const { abortCallback, eventCallback } = options ?? {};

    const abortController = new AbortController();
    if (abortCallback instanceof Function) {
      abortCallback(abortController);
    }

    let headers = options?.headers ?? { 'Content-Type': 'application/json' };
    const token = localStorage.getItem('aiter-token');
    if (token) {
      headers['Authorization'] = `Bearer ${token}`;
    }

    try {
      await fetchEventSource(this._baseUrl + url, {
        method: 'POST',
        headers,
        body: JSON.stringify(params),
        signal: abortController.signal,

        onmessage(event) {
          if (eventCallback instanceof Function) {
            eventCallback(event);
          }
        },

        onerror(err) {
          throw err;
        },
      });
    } catch (err) {
      if (err.status == 401) {
        const e = new Error(err.message);
        e.code = 'UNAUTHORIZED';
        throw e;
      } else {
        throw err;
      }
    }
  }
}

const client = new ApiClient();

export default client;
