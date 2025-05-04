import { useMemo } from 'react';
import { DndContext, PointerSensor, useSensor, useSensors } from '@dnd-kit/core';
import { SortableContext, useSortable, verticalListSortingStrategy } from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';
import {
  DocumentDTO,
  DocumentId,
  GetDocumentsResult,
  ProjectData,
  TaskData,
  TaskStatus,
} from 'dto';
import { Callback, cx, toSorted } from 'utils';
import { useShallowMemo } from 'utils/hooks';
import { useSuspenseQuery } from 'utils/suspense';
import { RPC } from 'utils/network';
import { CardContainer } from 'Workspace/CardContainer';
import { useCardContext } from 'Workspace/controller';
import { DropdownMenu, DropdownOptions } from 'components/DropdownMenu';
import { ProgressLocker } from 'components/ProgressLocker';
import { Markup } from 'components/Markup';
import { Button, IconButton } from 'components/Button';
import { Spoiler } from 'components/Spoiler';
import { Icon } from 'components/Icon';
import { DocumentTitle } from './DocumentTitle';
import { CONFLICT_INDICATOR, STAGED_INDICATOR } from './Indicators';

type TaskItemProps = {
  id: DocumentId;
  data: TaskData;
};
function TaskItem({ id, data }: TaskItemProps) {
  const { card, controller } = useCardContext();

  const {
    attributes, //
    listeners,
    setNodeRef,
    setActivatorNodeRef,
    transform,
    transition,
    isDragging,
    active,
  } = useSortable({ id });

  const style = {
    transform: CSS.Translate.toString(transform),
    transition,
  };

  return (
    <li
      ref={setNodeRef}
      {...attributes}
      style={style}
      className={cx(
        'pr-2 py-2 group hover:var-item-active-bg-color cursor-default font-medium touch-none relative flex flex-row items-center',
        data.status === 'Cancelled' && 'line-through',
      )}
      onClick={() => {
        controller.pushDocument(card.id, id);
      }}
    >
      <div
        ref={setActivatorNodeRef}
        className={cx('invisible', {
          'group-hover:visible': !active || isDragging,
          'cursor-grab': !isDragging,
          'cursor-grabbing': isDragging,
        })}
        {...listeners}
      >
        <Icon variant="drag" className="h-7 w-7" />
      </div>

      <div>{data.title}</div>

      <IconButton
        icon="link-arrow"
        size="sm"
        className="invisible group-hover:visible inline-block ml-3"
        onClick={(e) => {
          e.stopPropagation();
          controller.openDocument(id, true);
        }}
      />
    </li>
  );
}

type TaskGroupProps = {
  title: string;
  tasks: GetDocumentsResult<TaskData>[];
  defaultOpen?: boolean;
  sessionKey: string;
  onDrop: (taskId: DocumentId, targetId: DocumentId) => void;
};

function TaskGroup({ title, tasks, defaultOpen = false, sessionKey, onDrop }: TaskGroupProps) {
  const sensors = useSensors(useSensor(PointerSensor));

  const ids = tasks.map((task) => task.id);

  return (
    <Spoiler
      className="mb-4"
      heading={
        <h2 className="heading-2 text-base uppercase">
          {title} <span className="text-slate-300 ml-2">{tasks.length}</span>
        </h2>
      }
      sessionKey={sessionKey}
      open={defaultOpen}
    >
      <DndContext
        sensors={sensors}
        onDragEnd={(e) => {
          const activeId = e.active.id;
          const overId = e.over?.id;
          if (!overId || activeId === overId) {
            return;
          }

          onDrop(activeId as DocumentId, overId as DocumentId);
        }}
      >
        <SortableContext items={ids} strategy={verticalListSortingStrategy}>
          <ul>
            {tasks.map((task) => (
              <TaskItem key={task.id} id={task.id} data={task.data} />
            ))}
          </ul>
        </SortableContext>
      </DndContext>
    </Spoiler>
  );
}

function filterTasks(
  allTasks: GetDocumentsResult<TaskData>[],
  ...statuses: TaskStatus[]
): GetDocumentsResult<TaskData>[] {
  return allTasks.filter((task) => statuses.includes(task.data.status));
}

type ProjectCardProps = {
  document: DocumentDTO;
  isUpdating: boolean;
  onForceEditor: Callback;
  onAddTask: Callback;
  options: DropdownOptions;
};

export function ProjectCard({
  document,
  isUpdating,
  onForceEditor,
  onAddTask,
  options,
}: ProjectCardProps) {
  const projectData = document.data as ProjectData;
  const orderedTaskIds = projectData.tasks;
  const sortedTaskIds = useMemo(() => toSorted(orderedTaskIds), [orderedTaskIds]);

  const ids = useShallowMemo(sortedTaskIds);

  const {
    value: { documents },
  } = useSuspenseQuery({
    typeName: 'GetDocuments',
    ids,
  });

  const orderedTasks = toSorted(
    documents as GetDocumentsResult<TaskData>[],
    (a, b) => orderedTaskIds.indexOf(a.id) - orderedTaskIds.indexOf(b.id),
  );

  const tasksInProgress = filterTasks(orderedTasks, 'InProgress');
  const tasksTodo = filterTasks(orderedTasks, 'Todo');
  const tasksCompleted = filterTasks(orderedTasks, 'Done', 'Cancelled');

  const onDrop = (taskId: DocumentId, targetId: DocumentId) => {
    const newPos = orderedTaskIds.indexOf(targetId);
    if (newPos === -1) {
      throw new Error(`Can't find task ${targetId}`);
    }

    // TODO optimistic reorder

    void RPC.ReorderCollectionRefs({
      collectionId: document.id,
      id: taskId,
      newPos,
    });
  };

  return (
    <CardContainer
      leftToolbar={
        <>
          <DropdownMenu icon="dots-horizontal" align="bottom-left" options={options} />
          {document.isStaged && STAGED_INDICATOR}
          {document.hasConflict && CONFLICT_INDICATOR}
        </>
      }
      title={<DocumentTitle documentType={document.documentType} title={document.title} />}
      showTitleOnScroll
      rightToolbar={
        <>
          <Button leadingIcon="add-document" variant="simple" size="sm" onClick={onAddTask}>
            Add task
          </Button>

          <IconButton
            icon="pencil-square"
            size="lg"
            title="Open editor"
            onClick={onForceEditor}
            className="relative"
          />
        </>
      }
    >
      {isUpdating && <ProgressLocker />}

      <h1 className="heading-1 text-2xl mt-4 text-center text-sky-900 dark:text-sky-300 tracking-wider">
        {projectData.name}
      </h1>
      <div className="font-medium text-xs text-slate-400 uppercase tracking-wider text-center mb-8">
        PROJECT
      </div>

      <div className="mb-8">
        <Markup markup={projectData.description} />

        {document.backrefs.length > 0 && <div>{document.backrefs.length} backrefs</div>}
      </div>

      <TaskGroup
        title="In progress"
        tasks={tasksInProgress}
        defaultOpen
        sessionKey={`workspace-project-card-${document.id}-spoiler-in-progress`}
        onDrop={onDrop}
      />
      <TaskGroup
        title="Todo"
        tasks={tasksTodo}
        sessionKey={`workspace-project-card-${document.id}-spoiler-todo`}
        onDrop={onDrop}
      />
      <TaskGroup
        title="Completed"
        tasks={tasksCompleted}
        sessionKey={`workspace-project-card-${document.id}-spoiler-completed`}
        onDrop={onDrop}
      />
    </CardContainer>
  );
}
