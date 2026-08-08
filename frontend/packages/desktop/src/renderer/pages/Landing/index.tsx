import React from 'react';
import { useNavigate } from 'react-router-dom';
import { Button, Tag } from '@arco-design/web-react';
import {
  Brain,
  Shield,
  Robot,
  Branch,
  Local,
  Thunderbolt,
  Magic,
  Headset,
  Github,
  BookOne,
  Puzzle,
  ArrowRight,
} from '@icon-park/react';
import '@/renderer/pages/Landing/index.css';

interface FeatureCard {
  icon: React.ReactNode;
  title: string;
  desc: string;
  accent: string;
}

interface ScenarioCard {
  icon: React.ReactNode;
  title: string;
  desc: string;
}

const FEATURES: FeatureCard[] = [
  {
    icon: <Local theme='filled' size={26} />,
    title: '本地优先 · 数据不出本机',
    desc: '所有对话、知识库与 Agent 运行都在你自己的机器上完成。纯 Rust 后端 + 静态前端，零云端依赖，隐私天然可控。',
    accent: '#e8546b',
  },
  {
    icon: <Branch theme='filled' size={26} />,
    title: '多 Agent 编排',
    desc: '基于 rupoo 内核的工程化 Agent，可理解项目结构、匹配框架约定、自验编译与测试，多文件协同改动不跑偏。',
    accent: '#f0883e',
  },
  {
    icon: <Robot theme='filled' size={26} />,
    title: '托管主流模型 CLI',
    desc: '一键接入 Claude、Codex 及 DeepSeek、通义千问、智谱 GLM 等国产大模型，复用你本地已装的同一个 CLI，无需另装环境。',
    accent: '#d4609f',
  },
  {
    icon: <BookOne theme='filled' size={26} />,
    title: '知识库 Wiki',
    desc: 'Markdown 落盘 + SQLite FTS5 全文检索，拖入 PDF/DOCX/MD 自动切片，结构化标签与双链，让助手持续读懂你的资料。',
    accent: '#9b6dd6',
  },
  {
    icon: <Puzzle theme='filled' size={26} />,
    title: '开放生态 · MCP / Skills',
    desc: '内置技能市场与 MCP 工具协议，外接任意模型服务与第三方工具，能力像搭积木一样按需扩展。',
    accent: '#5b8def',
  },
  {
    icon: <Thunderbolt theme='filled' size={26} />,
    title: '开箱即用 · 单二进制',
    desc: '一个 Rust 二进制内嵌完整前端，浏览器直接打开即用；默认本地绑定，无需登录墙，下载即用、运行轻量。',
    accent: '#2bb6a3',
  },
];

const SCENARIOS: ScenarioCard[] = [
  {
    icon: <Magic theme='outline' size={22} />,
    title: '法律 / 教育 / 医疗 / 金融',
    desc: '面向垂直行业的解决方案模板，把行业知识灌进本地知识库，让 Agent 成为你的专属领域助理。',
  },
  {
    icon: <Headset theme='outline' size={22} />,
    title: '全天候个人助理',
    desc: '7×24 待命处理日常事务：写代码、整资料、跑任务、定计划，本地运行稳定可靠。',
  },
  {
    icon: <Local theme='outline' size={22} />,
    title: '隐私敏感型工作',
    desc: '合同、病历、财报等敏感资料全程不离开本机，企业内网亦可离线部署。',
  },
];

const Landing: React.FC = () => {
  const navigate = useNavigate();

  const handleEnter = () => {
    navigate('/guid');
  };

  return (
    <div className='rose-landing'>
      {/* 顶部导航 */}
      <header className='rose-nav'>
        <div className='rose-nav-inner'>
          <div className='rose-brand'>
            <div className='rose-brand-logo'>
              <Brain theme='filled' size={22} fill='#fff' />
            </div>
            <span className='rose-brand-name'>DoDidDoneUi</span>
            <Tag size='small' style={{ color: '#e8546b', background: 'rgba(232,84,107,0.12)', borderColor: 'transparent' }} className='rose-brand-tag'>
              本地优先 AI 工作台
            </Tag>
          </div>
          <nav className='rose-nav-links'>
            <a href='#features'>核心能力</a>
            <a href='#scenarios'>应用场景</a>
            <a href='#tech'>技术亮点</a>
            <Button type='primary' shape='round' onClick={handleEnter} className='rose-enter-btn'>
              进入工作台
              <ArrowRight theme='outline' size={14} />
            </Button>
          </nav>
        </div>
      </header>

      {/* Hero */}
      <section className='rose-hero'>
        <div className='rose-hero-glow' />
        <div className='rose-hero-inner'>
          <Tag size='large' style={{ color: '#e8546b', background: 'rgba(232,84,107,0.12)', borderColor: 'transparent' }} className='rose-hero-badge'>
            <Brain theme='outline' size={14} /> 你的本地多 Agent 编排平台
          </Tag>
          <h1 className='rose-hero-title'>
            把多个 AI Agent
            <br />
            装进你自己的机器
          </h1>
          <p className='rose-hero-sub'>
            DoDidDoneUi 是本地运行的多 Agent 编排平台：纯 Rust 后端 + 网页前端，数据不出本机。
            <br />
            接入 Claude / Codex / 国产大模型，编排工程化 Agent，构建专属知识库——打开即用。
          </p>
          <div className='rose-hero-actions'>
            <Button type='primary' size='large' shape='round' onClick={handleEnter} className='rose-hero-cta'>
              立即进入工作台
              <ArrowRight theme='outline' size={16} />
            </Button>
            <Button size='large' shape='round' className='rose-hero-ghost' href='#features'>
              了解能力
            </Button>
          </div>
          <div className='rose-hero-meta'>
            <span>
              <Local theme='outline' size={14} /> 本地运行
            </span>
            <span>
              <Github theme='outline' size={14} /> 开源派生 (Apache-2.0)
            </span>
            <span>
              <Thunderbolt theme='outline' size={14} /> 单二进制分发
            </span>
          </div>
        </div>
      </section>

      {/* 核心能力 */}
      <section id='features' className='rose-section'>
        <div className='rose-section-head'>
          <h2 className='rose-section-title'>核心能力</h2>
          <p className='rose-section-desc'>从底层运行时到上层生态，DoDidDoneUi 为本地 AI 工作流提供完整闭环。</p>
        </div>
        <div className='rose-feature-grid'>
          {FEATURES.map((f) => (
            <div className='rose-feature-card' key={f.title}>
              <div className='rose-feature-icon' style={{ color: f.accent, background: `${f.accent}1a` }}>
                {f.icon}
              </div>
              <h3 className='rose-feature-title'>{f.title}</h3>
              <p className='rose-feature-desc'>{f.desc}</p>
            </div>
          ))}
        </div>
      </section>

      {/* 应用场景 */}
      <section id='scenarios' className='rose-section rose-section-alt'>
        <div className='rose-section-head'>
          <h2 className='rose-section-title'>应用场景</h2>
          <p className='rose-section-desc'>无论个人提效还是行业落地，DoDidDoneUi 都能在本地稳稳接住。</p>
        </div>
        <div className='rose-scenario-grid'>
          {SCENARIOS.map((s) => (
            <div className='rose-scenario-card' key={s.title}>
              <div className='rose-scenario-icon'>{s.icon}</div>
              <h3 className='rose-scenario-title'>{s.title}</h3>
              <p className='rose-scenario-desc'>{s.desc}</p>
            </div>
          ))}
        </div>
      </section>

      {/* 技术亮点 */}
      <section id='tech' className='rose-section'>
        <div className='rose-section-head'>
          <h2 className='rose-section-title'>技术亮点</h2>
          <p className='rose-section-desc'>为长期自托管与工程化使用而设计。</p>
        </div>
        <div className='rose-tech-grid'>
          <div className='rose-tech-item'>
            <Robot theme='filled' size={20} />
            <div>
              <h4>rupoo 工程化内核</h4>
              <p>理解工程结构、框架约定与跨文件接口，改动后自验编译测试。</p>
            </div>
          </div>
          <div className='rose-tech-item'>
            <Puzzle theme='filled' size={20} />
            <div>
              <h4>模型与协议解耦</h4>
              <p>统一接入主流大模型 CLI 与 OpenAI/Anthropic 协议，换模型不改工作流。</p>
            </div>
          </div>
          <div className='rose-tech-item'>
            <BookOne theme='filled' size={20} />
            <div>
              <h4>可解释检索</h4>
              <p>FTS5 全文检索为唯一基础层，确定性、零依赖、结果可追溯。</p>
            </div>
          </div>
          <div className='rose-tech-item'>
            <Shield theme='filled' size={20} />
            <div>
              <h4>安全默认</h4>
              <p>默认绑定 127.0.0.1、JWT+CSRF、Host 校验、WS Origin 校验，遥测默认关闭。</p>
            </div>
          </div>
        </div>
      </section>

      {/* 底部 CTA */}
      <section className='rose-cta'>
        <div className='rose-cta-inner'>
          <h2 className='rose-cta-title'>准备好把 AI 带回本地了吗？</h2>
          <p className='rose-cta-sub'>打开 DoDidDoneUi，几分钟即可拥有属于你自己的多 Agent 工作台。</p>
          <Button type='primary' size='large' shape='round' onClick={handleEnter} className='rose-hero-cta'>
            进入工作台
            <ArrowRight theme='outline' size={16} />
          </Button>
        </div>
      </section>

      <footer className='rose-footer'>
        <span>
          DoDidDoneUi · 本地优先的多 Agent 编排平台 · 派生自 AionUi (Apache-2.0)
        </span>
      </footer>
    </div>
  );
};

export default Landing;
