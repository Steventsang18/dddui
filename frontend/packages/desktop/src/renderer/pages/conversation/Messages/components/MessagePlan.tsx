import { Badge } from '@arco-design/web-react';
import { IconCheckCircle, IconDown, IconRight } from '@arco-design/web-react/icon';
import React, { useState } from 'react';
import type { IMessagePlan } from '@/common/chat/chatLib';

const MessagePlan: React.FC<{ message: IMessagePlan }> = ({ message }) => {
  const [showMore, setShowMore] = useState(true);
  return (
    <div>
      <div className='flex items-center gap-10px text-t-tertiary cursor-pointer' onClick={() => setShowMore(!showMore)}>
        <Badge status='default' text='To do list' className={'![&_span.arco-badge-status-text]:text-t-tertiary'}></Badge>
        {showMore ? <IconDown /> : <IconRight />}
      </div>
      {showMore && (
        <div className='p-l-20px flex flex-col gap-8px pt-8px'>
          {message.content.entries.map((item, index) => {
            return (
              <div className='flex flex-row items-center text-t-tertiary gap-8px'>
                {item.status === 'completed' ? (
                  <IconCheckCircle fontSize={22} strokeWidth={4} className='flex text-success' />
                ) : (
                  <div className='size-22px flex items-center justify-center'>
                    <div className='size-14px  rd-10px b-2px b-solid b-[rgba(201,205,212,1)]'></div>
                  </div>
                )}
                <span>{item.content} </span>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
};

export default MessagePlan;
