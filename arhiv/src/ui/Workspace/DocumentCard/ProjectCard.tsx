import { DocumentDTO, ListDocumentsResult, ProjectData, TaskData, TaskStatus } from 'dto';
import { Callback, copyTextToClipbard, cx, getDocumentUrl } from 'utils';
import { useShallowMemo } from 'utils/hooks';
import { useSuspenseQuery } from 'utils/suspense';
import { DropdownMenu } from 'components/DropdownMenu';
import { CardContainer } from 'Workspace/CardContainer';
import { ProgressLocker } from 'components/ProgressLocker';
import { Markup } from 'components/Markup';
import { IconButton } from 'components/Button';
import { Spoiler } from 'components/Spoiler';
import { useCardContext } from 'Workspace/workspace-reducer';

type TaskGroupProps = {
  title: string;
  tasks: ListDocumentsResult<TaskData>[];
  defaultOpen?: boolean;
  sessionKey: string;
};

function TaskGroup({ title, tasks, defaultOpen = false, sessionKey }: TaskGroupProps) {
  const { card, actions } = useCardContext();

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
      <ul className="list-disc list-inside marker:text-slate-400">
        {tasks.map((task) => (
          <li
            key={task.id}
            className={cx(
              'p-2 group hover:bg-amber-100 cursor-pointer font-medium',
              task.data.status === 'Cancelled' && 'line-through',
            )}
            onClick={() => {
              actions.pushDocument(card.id, task.id);
            }}
          >
            {task.data.title}

            <IconButton
              icon="link-arrow"
              size="sm"
              className="invisible group-hover:visible inline-block ml-3"
              onClick={(e) => {
                e.stopPropagation();
                actions.openDocument(task.id, true);
              }}
            />
          </li>
        ))}
      </ul>
    </Spoiler>
  );
}

function filterTasks(
  allTasks: ListDocumentsResult<TaskData>[],
  ...statuses: TaskStatus[]
): ListDocumentsResult<TaskData>[] {
  return allTasks.filter((task) => statuses.includes(task.data.status));
}

type ProjectCardProps = {
  document: DocumentDTO;
  isUpdating: boolean;
  onForceEditor: Callback;
};

export function ProjectCard({ document, isUpdating, onForceEditor }: ProjectCardProps) {
  const projectData = document.data as ProjectData;

  const ids = useShallowMemo(projectData.tasks);

  const {
    value: { documents },
  } = useSuspenseQuery({
    typeName: 'GetDocuments',
    ids,
  });

  const tasks = documents as ListDocumentsResult<TaskData>[];

  const tasksInProgress = filterTasks(tasks, 'InProgress');
  const tasksTodo = filterTasks(tasks, 'Todo');
  const tasksCompleted = filterTasks(tasks, 'Done', 'Cancelled');

  return (
    <CardContainer
      leftToolbar={
        <>
          <DropdownMenu
            icon="dots-horizontal"
            align="bottom-left"
            options={[
              {
                text: `ID ${document.id}`,
                icon: 'clipboard',
                onClick: () => {
                  void copyTextToClipbard(document.id);
                },
              },
              {
                text: 'Copy link',
                icon: 'clipboard',
                onClick: () => {
                  void copyTextToClipbard(getDocumentUrl(document.id));
                },
              },
            ]}
          />

          <span className="font-medium text-xs text-slate-400 uppercase tracking-wider">
            PROJECT
            {document.backrefs.length > 0 && `${document.backrefs.length} backrefs`}
          </span>
        </>
      }
      rightToolbar={
        <IconButton
          icon="pencil-square"
          size="lg"
          title="Open editor"
          onClick={onForceEditor}
          className="relative"
        />
      }
    >
      {isUpdating && <ProgressLocker />}

      <h1 className="heading-1 mb-8 mt-8 text-center text-sky-900 tracking-wider">
        {projectData.name}
      </h1>
      <div className="mb-8">
        <Markup markup={projectData.description} />
      </div>

      <TaskGroup
        title="In progress"
        tasks={tasksInProgress}
        defaultOpen
        sessionKey={`workspace-project-card-${document.id}-spoiler-in-progress`}
      />
      <TaskGroup
        title="Todo"
        tasks={tasksTodo}
        sessionKey={`workspace-project-card-${document.id}-spoiler-todo`}
      />
      <TaskGroup
        title="Completed"
        tasks={tasksCompleted}
        sessionKey={`workspace-project-card-${document.id}-spoiler-completed`}
      />
    </CardContainer>
  );
}
