class GameStreamViewer {
    constructor() {
        this.peerConnection = null;
        this.websocket = null;
        this.isConnected = false;
        this.streamKey = '';
        this.serverUrl = '';
        this.protocol = 'webrtc';
        
        // 统计信息
        this.stats = {
            viewerCount: 0,
            latency: 0,
            bitrate: 0
        };
        
        // DOM 元素
        this.elements = {
            streamKey: document.getElementById('streamKey'),
            serverUrl: document.getElementById('serverUrl'),
            protocol: document.getElementById('protocol'),
            connectBtn: document.getElementById('connectBtn'),
            videoElement: document.getElementById('videoElement'),
            videoOverlay: document.getElementById('videoOverlay'),
            connectionStatus: document.getElementById('connectionStatus'),
            viewerCount: document.getElementById('viewerCount'),
            latency: document.getElementById('latency'),
            bitrate: document.getElementById('bitrate'),
            logs: document.getElementById('logs')
        };
        
        this.initializeEventListeners();
        this.log('info', '观看端初始化完成');
    }
    
    initializeEventListeners() {
        // 协议切换
        this.elements.protocol.addEventListener('change', (e) => {
            this.protocol = e.target.value;
            this.log('info', `切换到 ${this.protocol.toUpperCase()} 协议`);
        });
        
        // 视频元素事件
        this.elements.videoElement.addEventListener('loadstart', () => {
            this.log('info', '开始加载视频流');
            this.showVideoOverlay('加载中...');
        });
        
        this.elements.videoElement.addEventListener('loadeddata', () => {
            this.log('info', '视频数据已加载');
            this.hideVideoOverlay();
        });
        
        this.elements.videoElement.addEventListener('error', (e) => {
            this.log('error', `视频播放错误: ${e.message || '未知错误'}`);
            this.showVideoOverlay('视频播放错误');
        });
        
        this.elements.videoElement.addEventListener('waiting', () => {
            this.showVideoOverlay('缓冲中...');
        });
        
        this.elements.videoElement.addEventListener('playing', () => {
            this.hideVideoOverlay();
            this.log('info', '视频开始播放');
        });
    }
    
    async toggleConnection() {
        if (this.isConnected) {
            await this.disconnect();
        } else {
            await this.connect();
        }
    }
    
    async connect() {
        this.streamKey = this.elements.streamKey.value.trim();
        this.serverUrl = this.elements.serverUrl.value.trim();
        
        if (!this.streamKey) {
            this.log('error', '请输入直播流密钥');
            return;
        }
        
        if (!this.serverUrl) {
            this.log('error', '请输入服务器地址');
            return;
        }
        
        try {
            this.updateConnectionStatus('connecting');
            this.elements.connectBtn.disabled = true;
            
            if (this.protocol === 'webrtc') {
                await this.connectWebRTC();
            } else if (this.protocol === 'hls') {
                await this.connectHLS();
            }
            
        } catch (error) {
            this.log('error', `连接失败: ${error.message}`);
            this.updateConnectionStatus('disconnected');
            this.elements.connectBtn.disabled = false;
        }
    }
    
    async connectWebRTC() {
        this.log('info', '正在建立 WebRTC 连接...');
        
        // 创建 WebSocket 连接
        const wsUrl = this.serverUrl.replace('http', 'ws') + '/api/webrtc/ws';
        this.websocket = new WebSocket(wsUrl);
        
        this.websocket.onopen = () => {
            this.log('info', 'WebSocket 连接已建立');
            this.initializeWebRTC();
        };
        
        this.websocket.onmessage = (event) => {
            this.handleWebSocketMessage(event.data);
        };
        
        this.websocket.onerror = (error) => {
            this.log('error', `WebSocket 错误: ${error.message || '连接失败'}`);
        };
        
        this.websocket.onclose = () => {
            this.log('info', 'WebSocket 连接已关闭');
            this.updateConnectionStatus('disconnected');
        };
    }
    
    async initializeWebRTC() {
        try {
            // 创建 RTCPeerConnection
            this.peerConnection = new RTCPeerConnection({
                iceServers: [
                    { urls: 'stun:stun.l.google.com:19302' },
                    { urls: 'stun:stun1.l.google.com:19302' }
                ]
            });
            
            // 处理接收到的媒体流
            this.peerConnection.ontrack = (event) => {
                this.log('info', '接收到媒体流');
                const [remoteStream] = event.streams;
                this.elements.videoElement.srcObject = remoteStream;
                this.updateConnectionStatus('connected');
            };
            
            // 处理 ICE 候选
            this.peerConnection.onicecandidate = (event) => {
                if (event.candidate) {
                    this.sendSignal({
                        IceCandidate: {
                            candidate: event.candidate.candidate,
                            sdp_mid: event.candidate.sdpMid,
                            sdp_mline_index: event.candidate.sdpMLineIndex
                        }
                    });
                }
            };
            
            // 处理连接状态变化
            this.peerConnection.onconnectionstatechange = () => {
                this.log('debug', `WebRTC 连接状态: ${this.peerConnection.connectionState}`);
                
                if (this.peerConnection.connectionState === 'connected') {
                    this.isConnected = true;
                    this.updateConnectionStatus('connected');
                    this.elements.connectBtn.textContent = '断开';
                    this.elements.connectBtn.disabled = false;
                    this.startStatsCollection();
                } else if (this.peerConnection.connectionState === 'disconnected' || 
                          this.peerConnection.connectionState === 'failed') {
                    this.handleDisconnection();
                }
            };
            
            // 创建 Offer
            const offer = await this.peerConnection.createOffer({
                offerToReceiveVideo: true,
                offerToReceiveAudio: true
            });
            
            await this.peerConnection.setLocalDescription(offer);
            
            // 发送 Offer (使用正确的格式)
            this.sendSignal({
                Offer: {
                    stream_key: this.streamKey,
                    sdp: offer.sdp
                }
            });
            
        } catch (error) {
            this.log('error', `WebRTC 初始化失败: ${error.message}`);
            throw error;
        }
    }
    
    async connectHLS() {
        this.log('info', '正在连接 HLS 流...');
        
        const hlsUrl = `${this.serverUrl.replace('ws', 'http')}/hls/${this.streamKey}/playlist.m3u8`;
        
        if (this.elements.videoElement.canPlayType('application/vnd.apple.mpegurl')) {
            // 原生 HLS 支持 (Safari)
            this.elements.videoElement.src = hlsUrl;
            this.updateConnectionStatus('connected');
            this.isConnected = true;
            this.elements.connectBtn.textContent = '断开';
            this.elements.connectBtn.disabled = false;
        } else {
            // 使用 hls.js (其他浏览器)
            if (typeof Hls !== 'undefined' && Hls.isSupported()) {
                const hls = new Hls();
                hls.loadSource(hlsUrl);
                hls.attachMedia(this.elements.videoElement);
                
                hls.on(Hls.Events.MANIFEST_PARSED, () => {
                    this.log('info', 'HLS 清单解析完成');
                    this.updateConnectionStatus('connected');
                    this.isConnected = true;
                    this.elements.connectBtn.textContent = '断开';
                    this.elements.connectBtn.disabled = false;
                });
                
                hls.on(Hls.Events.ERROR, (event, data) => {
                    this.log('error', `HLS 错误: ${data.details}`);
                });
                
                this.hlsInstance = hls;
            } else {
                throw new Error('浏览器不支持 HLS 播放');
            }
        }
    }
    
    sendSignal(signal) {
        if (this.websocket && this.websocket.readyState === WebSocket.OPEN) {
            this.websocket.send(JSON.stringify(signal));
            this.log('debug', `发送信令: ${signal.type}`);
        }
    }
    
    async handleWebSocketMessage(data) {
        try {
            const signal = JSON.parse(data);
            this.log('debug', `接收信令: ${JSON.stringify(signal)}`);

            if (signal.Answer) {
                await this.peerConnection.setRemoteDescription({
                    type: 'answer',
                    sdp: signal.Answer.sdp
                });
                this.log('info', '设置远程描述完成');
            } else if (signal.IceCandidate) {
                await this.peerConnection.addIceCandidate({
                    candidate: signal.IceCandidate.candidate,
                    sdpMid: signal.IceCandidate.sdp_mid,
                    sdpMLineIndex: signal.IceCandidate.sdp_mline_index
                });
                this.log('debug', '添加 ICE 候选完成');
            } else if (signal.Error) {
                this.log('error', `服务器错误: ${signal.Error.message}`);
            }
        } catch (error) {
            this.log('error', `处理信令消息失败: ${error.message}`);
        }
    }
    
    async disconnect() {
        this.log('info', '正在断开连接...');
        
        this.isConnected = false;
        this.stopStatsCollection();
        
        if (this.peerConnection) {
            this.peerConnection.close();
            this.peerConnection = null;
        }
        
        if (this.websocket) {
            this.websocket.close();
            this.websocket = null;
        }
        
        if (this.hlsInstance) {
            this.hlsInstance.destroy();
            this.hlsInstance = null;
        }
        
        this.elements.videoElement.srcObject = null;
        this.elements.videoElement.src = '';
        
        this.updateConnectionStatus('disconnected');
        this.elements.connectBtn.textContent = '连接';
        this.elements.connectBtn.disabled = false;
        
        this.showVideoOverlay('已断开连接');
        this.log('info', '连接已断开');
    }
    
    handleDisconnection() {
        if (this.isConnected) {
            this.log('info', '连接意外断开，尝试重连...');
            setTimeout(() => {
                if (!this.isConnected) {
                    this.connect();
                }
            }, 3000);
        }
    }
    
    startStatsCollection() {
        if (this.protocol === 'webrtc' && this.peerConnection) {
            this.statsInterval = setInterval(async () => {
                try {
                    const stats = await this.peerConnection.getStats();
                    this.processWebRTCStats(stats);
                } catch (error) {
                    this.log('debug', `获取统计信息失败: ${error.message}`);
                }
            }, 1000);
        }
    }
    
    stopStatsCollection() {
        if (this.statsInterval) {
            clearInterval(this.statsInterval);
            this.statsInterval = null;
        }
    }
    
    processWebRTCStats(stats) {
        let inboundRtp = null;
        
        stats.forEach(report => {
            if (report.type === 'inbound-rtp' && report.mediaType === 'video') {
                inboundRtp = report;
            }
        });
        
        if (inboundRtp) {
            // 计算码率
            if (this.lastStats && this.lastStats.bytesReceived) {
                const bytesDiff = inboundRtp.bytesReceived - this.lastStats.bytesReceived;
                const timeDiff = inboundRtp.timestamp - this.lastStats.timestamp;
                const bitrate = Math.round((bytesDiff * 8) / (timeDiff / 1000) / 1000); // kbps
                
                this.updateStats({
                    bitrate: bitrate > 0 ? `${bitrate} kbps` : '-'
                });
            }
            
            this.lastStats = {
                bytesReceived: inboundRtp.bytesReceived,
                timestamp: inboundRtp.timestamp
            };
        }
    }
    
    updateConnectionStatus(status) {
        const statusElement = this.elements.connectionStatus;
        statusElement.className = 'connection-status';
        
        switch (status) {
            case 'disconnected':
                statusElement.classList.add('status-disconnected');
                statusElement.textContent = '未连接';
                break;
            case 'connecting':
                statusElement.classList.add('status-connecting');
                statusElement.textContent = '连接中...';
                break;
            case 'connected':
                statusElement.classList.add('status-connected');
                statusElement.textContent = '已连接';
                break;
        }
    }
    
    updateStats(stats) {
        if (stats.viewerCount !== undefined) {
            this.elements.viewerCount.textContent = stats.viewerCount;
        }
        if (stats.latency !== undefined) {
            this.elements.latency.textContent = stats.latency;
        }
        if (stats.bitrate !== undefined) {
            this.elements.bitrate.textContent = stats.bitrate;
        }
    }
    
    showVideoOverlay(message) {
        this.elements.videoOverlay.textContent = message;
        this.elements.videoOverlay.style.display = 'block';
    }
    
    hideVideoOverlay() {
        this.elements.videoOverlay.style.display = 'none';
    }
    
    log(level, message) {
        const timestamp = new Date().toLocaleTimeString();
        const logEntry = document.createElement('div');
        logEntry.className = 'log-entry';
        
        logEntry.innerHTML = `
            <span class="log-timestamp">[${timestamp}]</span>
            <span class="log-${level}">${message}</span>
        `;
        
        this.elements.logs.appendChild(logEntry);
        this.elements.logs.scrollTop = this.elements.logs.scrollHeight;
        
        // 限制日志条数
        while (this.elements.logs.children.length > 100) {
            this.elements.logs.removeChild(this.elements.logs.firstChild);
        }
        
        // 同时输出到控制台
        console[level] ? console[level](message) : console.log(message);
    }
}

// 全局实例
let viewer;

// 页面加载完成后初始化
document.addEventListener('DOMContentLoaded', () => {
    viewer = new GameStreamViewer();
});

// 全局函数供 HTML 调用
function toggleConnection() {
    if (viewer) {
        viewer.toggleConnection();
    }
}
